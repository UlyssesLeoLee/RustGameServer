import { useCallback, useEffect, useMemo, useState } from 'react';
import axios from 'axios';
import { Link } from 'react-router-dom';
import ActiveUsersChart from '../components/ActiveUsersChart';

interface Props {
  auth: string;
}

interface BroadcastEntry {
  id: number;
  message: string;
  admin: string;
  createdAt: string;
}

interface ServerEntry {
  id: string;
  name: string;
  region?: string;
  status: 'running' | 'stopped';
  onlinePlayers: number;
  lastUpdated?: string;
}

interface SummaryResponse {
  playerStats: {
    total: number;
    online: number;
    offline: number;
    banned: number;
    averageLevel: number;
    highValue: number;
  };
  activity: {
    totalBroadcasts: number;
    recentBroadcasts: BroadcastEntry[];
    totalGrants: number;
  };
  support: {
    open: number;
    total: number;
  };
  mall: {
    totalItems: number;
  };
  servers?: {
    stats: {
      total: number;
      running: number;
    };
    list?: ServerEntry[];
  };
}

export default function Dashboard({ auth }: Props) {
  const [message, setMessage] = useState('');
  const [status, setStatus] = useState('');
  const [statusLevel, setStatusLevel] = useState<
    'success' | 'warning' | 'error' | ''
  >('');
  const [summary, setSummary] = useState<SummaryResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [broadcasts, setBroadcasts] = useState<BroadcastEntry[]>([]);
  const [lastRefreshed, setLastRefreshed] = useState<Date | null>(null);

  const requestConfig = useMemo(
    () => ({ headers: { Authorization: `Bearer ${auth}` } }),
    [auth]
  );

  const fetchSummary = useCallback(async () => {
    setLoading(true);
    try {
      const res = await axios.get<SummaryResponse>(
        '/gm/summary',
        requestConfig
      );
      setSummary(res.data);
      setBroadcasts(res.data.activity?.recentBroadcasts || []);
      setError('');
      setLastRefreshed(new Date());
    } catch {
      setError('Failed to load overview metrics');
    } finally {
      setLoading(false);
    }
  }, [requestConfig]);

  useEffect(() => {
    fetchSummary();
  }, [fetchSummary]);

  useEffect(() => {
    if (!auth) return;
    const source = new EventSource(`/gm/events?token=${encodeURIComponent(auth)}`);
    source.onmessage = event => {
      try {
        const data: BroadcastEntry = JSON.parse(event.data);
        setBroadcasts(prev => {
          const next = [data, ...prev.filter(item => item.id !== data.id)];
          return next.slice(0, 10);
        });
        setSummary(prev => {
          if (!prev) return prev;
          const recent = [data, ...prev.activity.recentBroadcasts.filter(item => item.id !== data.id)].slice(0, 5);
          return {
            ...prev,
            activity: {
              ...prev.activity,
              totalBroadcasts: prev.activity.totalBroadcasts + 1,
              recentBroadcasts: recent,
            },
          };
        });
      } catch {
        // ignore invalid payloads
      }
    };
    source.onerror = () => {
      source.close();
    };
    return () => {
      source.close();
    };
  }, [auth]);

  const broadcast = async () => {
    if (!message.trim()) {
      setStatus('Please enter a broadcast message');
      setStatusLevel('warning');
      return;
    }
    try {
      const res = await axios.post('/gm/broadcast', { message }, requestConfig);
      setStatus(res.data.status || 'ok');
      setStatusLevel('success');
      setMessage('');
      await fetchSummary();
    } catch {
      setStatus('Failed to send broadcast');
      setStatusLevel('error');
    }
  };

  const serverList = useMemo(() => {
    if (!summary?.servers?.list) return [];
    return summary.servers.list
      .slice()
      .sort((a, b) => (b.onlinePlayers || 0) - (a.onlinePlayers || 0))
      .slice(0, 6);
  }, [summary]);

  const offlineServers = useMemo(() => {
    if (!summary?.servers?.stats) return 0;
    return Math.max(
      0,
      (summary.servers.stats.total || 0) - (summary.servers.stats.running || 0)
    );
  }, [summary]);

  const statusClass = useMemo(() => {
    if (!statusLevel) return 'status-message';
    return `status-message status-${statusLevel}`;
  }, [statusLevel]);

  return (
    <div>
      <section className="section">
        <div className="section-header">
          <h2>Welcome to the GM Dashboard</h2>
          <button onClick={fetchSummary} disabled={loading}>
            {loading ? 'Refreshing…' : 'Refresh metrics'}
          </button>
        </div>
        <p className="section-description">
          Monitor player activity, server health and key operations metrics in real
          time.
        </p>
        {error && <p className="error">{error}</p>}
        <div className="dashboard-grid">
          <div className="metric-card">
            <span>Total Players</span>
            <strong>{summary?.playerStats.total ?? '—'}</strong>
            <small>
              Online {summary?.playerStats.online ?? 0} · Banned{' '}
              {summary?.playerStats.banned ?? 0}
            </small>
          </div>
          <div className="metric-card">
            <span>Average Level</span>
            <strong>{summary?.playerStats.averageLevel ?? '—'}</strong>
            <small>
              High value players: {summary?.playerStats.highValue ?? 0}
            </small>
          </div>
          <div className="metric-card">
            <span>Active Servers</span>
            <strong>{summary?.servers?.stats.running ?? 0}</strong>
            <small>Total servers: {summary?.servers?.stats.total ?? 0}</small>
          </div>
          <div className="metric-card">
            <span>Open Tickets</span>
            <strong>{summary?.support.open ?? 0}</strong>
            <small>Total tickets: {summary?.support.total ?? 0}</small>
          </div>
          <div className="metric-card">
            <span>Mall Items</span>
            <strong>{summary?.mall.totalItems ?? 0}</strong>
            <small>Grants issued: {summary?.activity.totalGrants ?? 0}</small>
          </div>
          <div className="metric-card">
            <span>Broadcasts Sent</span>
            <strong>{summary?.activity.totalBroadcasts ?? 0}</strong>
            <small>Live feed updates automatically</small>
          </div>
        </div>
        {summary && (
          <div className="chips">
            <span className="chip">
              Online players {summary.playerStats.online ?? 0}
            </span>
            <span className="chip">
              Average level {summary.playerStats.averageLevel ?? '—'}
            </span>
            <span className="chip">
              Servers offline {offlineServers}
            </span>
          </div>
        )}
        {lastRefreshed && (
          <p className="muted">
            Last refreshed at {lastRefreshed.toLocaleTimeString()}
          </p>
        )}
      </section>
      <section className="section">
        <h3>Quick Navigation</h3>
        <p className="section-description">
          Jump directly to player, economy and support modules for deeper
          operations.
        </p>
        <div className="menu-grid">
          <Link to="/players">Players</Link>
          <Link to="/servers">Servers</Link>
          <Link to="/items">Items</Link>
          <Link to="/mall">Mall</Link>
          <Link to="/reports">Reports</Link>
          <Link to="/support">Support</Link>
        </div>
      </section>
      {serverList.length > 0 && (
        <section className="section">
          <div className="section-header">
            <h3>Live Server Snapshot</h3>
            <span className="muted">
              {summary?.servers?.stats.running ?? 0} of{' '}
              {summary?.servers?.stats.total ?? 0} running
            </span>
          </div>
          <div className="table-wrapper">
            <table className="table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Name</th>
                  <th>Region</th>
                  <th>Status</th>
                  <th>Online Players</th>
                  <th>Last Updated</th>
                </tr>
              </thead>
              <tbody>
                {serverList.map(server => (
                  <tr key={server.id}>
                    <td>{server.id}</td>
                    <td>{server.name}</td>
                    <td>{server.region || '—'}</td>
                    <td>
                      <span
                        className={`status-tag ${
                          server.status === 'running'
                            ? 'status-online'
                            : 'status-offline'
                        }`}
                      >
                        {server.status}
                      </span>
                    </td>
                    <td>{server.onlinePlayers ?? 0}</td>
                    <td>
                      {server.lastUpdated
                        ? new Date(server.lastUpdated).toLocaleString()
                        : '—'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      )}
      <section className="section">
        <ActiveUsersChart title="Active Users (last samples)" authToken={auth} />
      </section>
      <section className="section">
        <div className="section-header">
          <h3>Broadcast Message</h3>
          <button
            className="button-secondary"
            onClick={() => {
              setMessage('');
              setStatus('');
              setStatusLevel('');
            }}
          >
            Clear
          </button>
        </div>
        <p className="section-description">
          Send a global announcement that will immediately appear for online
          players.
        </p>
        <input
          placeholder="Message"
          value={message}
          onChange={e => setMessage(e.target.value)}
        />
        <button onClick={broadcast}>Send broadcast</button>
        {status && <p className={statusClass}>{status}</p>}
      </section>
      <section className="section">
        <div className="section-header">
          <h3>Recent Broadcasts</h3>
          <span className="muted">
            Total {summary?.activity.totalBroadcasts ?? 0} broadcasts
          </span>
        </div>
        <ul className="list">
          {broadcasts.length === 0 && <li className="empty-state">No broadcasts yet.</li>}
          {broadcasts.map(entry => (
            <li key={entry.id}>
              <div className="list-title">{entry.message}</div>
              <div className="list-meta">
                <span>by {entry.admin}</span>
                <span>{new Date(entry.createdAt).toLocaleString()}</span>
              </div>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
