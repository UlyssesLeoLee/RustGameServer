import { useCallback, useEffect, useMemo, useState } from 'react';
import axios from 'axios';

interface Props {
  auth: string;
}

export default function Servers({ auth }: Props) {
  interface ServerStatus {
    id: string;
    name: string;
    region: string;
    status: 'running' | 'stopped';
    onlinePlayers: number;
    lastUpdated: string;
  }

  const [servers, setServers] = useState<ServerStatus[]>([]);
  const [statusMessage, setStatusMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const [statusLevel, setStatusLevel] = useState<'success' | 'error' | ''>('');
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  const requestConfig = useMemo(
    () => ({ headers: { Authorization: `Bearer ${auth}` } }),
    [auth]
  );

  const fetchServers = useCallback(async () => {
    try {
      const res = await axios.get<{ servers: ServerStatus[] }>(
        '/gm/servers',
        requestConfig
      );
      setServers(res.data.servers || []);
      setStatusMessage('');
      setStatusLevel('');
      setLastUpdated(new Date());
    } catch {
      setStatusMessage('Failed to fetch server status');
      setStatusLevel('error');
    }
  }, [requestConfig]);

  useEffect(() => {
    fetchServers();
  }, [fetchServers]);

  useEffect(() => {
    if (!autoRefresh) return;
    const id = setInterval(fetchServers, 10000);
    return () => clearInterval(id);
  }, [autoRefresh, fetchServers]);

  const performAction = async (id: string, action: 'start' | 'stop') => {
    setLoading(true);
    try {
      const res = await axios.post(
        `/gm/servers/${id}/${action}`,
        {},
        requestConfig
      );
      const serverName = res.data.server?.name || id;
      setStatusMessage(`${res.data.status || action} ${serverName}`);
      setStatusLevel('success');
      await fetchServers();
    } catch {
      setStatusMessage(`Failed to ${action} server ${id}`);
      setStatusLevel('error');
    } finally {
      setLoading(false);
    }
  };

  const runningServers = useMemo(
    () => servers.filter(server => server.status === 'running').length,
    [servers]
  );

  const totalPlayers = useMemo(
    () => servers.reduce((acc, server) => acc + (server.onlinePlayers || 0), 0),
    [servers]
  );

  const statusClass = statusLevel
    ? `status-message status-${statusLevel}`
    : 'status-message';

  return (
    <div>
      <section className="section">
        <div className="section-header">
          <h2>Server Management</h2>
          <div>
            <label className="muted" style={{ marginRight: '12px' }}>
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={event => setAutoRefresh(event.target.checked)}
                style={{ marginRight: '6px' }}
              />
              Auto refresh
            </label>
            <button onClick={fetchServers} disabled={loading}>
              {loading ? 'Refreshing…' : 'Refresh'}
            </button>
          </div>
        </div>
        <p className="section-description">
          Monitor shard availability and take real-time action when anomalies are
          detected.
        </p>
        <div className="summary-grid">
          <div className="summary-card">
            <span>Total servers</span>
            <strong>{servers.length}</strong>
            <small>Running {runningServers}</small>
          </div>
          <div className="summary-card">
            <span>Players online</span>
            <strong>{totalPlayers}</strong>
            <small>Across all running shards</small>
          </div>
          <div className="summary-card">
            <span>Offline servers</span>
            <strong>{Math.max(0, servers.length - runningServers)}</strong>
            <small>Requires manual intervention</small>
          </div>
        </div>
        {lastUpdated && (
          <p className="muted">Last synced at {lastUpdated.toLocaleTimeString()}</p>
        )}
        {statusMessage && <p className={statusClass}>{statusMessage}</p>}
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
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {servers.length === 0 && (
                <tr>
                  <td colSpan={7} className="empty-state">
                    No server information
                  </td>
                </tr>
              )}
              {servers.map(server => (
                <tr key={server.id}>
                  <td>{server.id}</td>
                  <td>{server.name}</td>
                  <td>{server.region}</td>
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
                  <td>{server.onlinePlayers}</td>
                  <td>{new Date(server.lastUpdated).toLocaleString()}</td>
                  <td className="table-actions">
                    <button
                      onClick={() => performAction(server.id, 'start')}
                      disabled={loading || server.status === 'running'}
                    >
                      Start
                    </button>
                    <button
                      onClick={() => performAction(server.id, 'stop')}
                      disabled={loading || server.status === 'stopped'}
                    >
                      Stop
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
