import { FormEvent, useCallback, useEffect, useMemo, useState } from 'react';
import axios from 'axios';

interface Props {
  auth: string;
}

export default function Players({ auth }: Props) {
  interface Player {
    id: number;
    name: string;
    level: number;
    vipLevel: number;
    status: 'online' | 'offline' | 'banned';
    lastLogin: string;
    totalSpend: number;
    region?: string;
    guild?: string;
  }

  interface PlayerStats {
    total: number;
    online: number;
    offline: number;
    banned: number;
    averageLevel: number;
    highValue: number;
  }

  interface PlayersResponse {
    players: Player[];
    total: number;
    page: number;
    pageSize: number;
    totalPages: number;
    stats: {
      overall: PlayerStats;
      filtered: PlayerStats;
    };
  }

  const [players, setPlayers] = useState<Player[]>([]);
  const [stats, setStats] = useState<PlayersResponse['stats'] | null>(null);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);
  const [total, setTotal] = useState(0);
  const [search, setSearch] = useState('');
  const [appliedSearch, setAppliedSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<'all' | 'online' | 'offline' | 'banned'>('all');
  const [sortBy, setSortBy] = useState<'lastLogin' | 'level' | 'name' | 'totalSpend'>('lastLogin');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [selectedPlayer, setSelectedPlayer] = useState<Player | null>(null);

  const requestConfig = useMemo(
    () => ({ headers: { Authorization: `Bearer ${auth}` } }),
    [auth]
  );

  const loadPlayers = useCallback(
    async (pageToLoad: number, pageSizeValue: number) => {
      setLoading(true);
      try {
        const res = await axios.get<PlayersResponse>('/gm/players', {
          ...requestConfig,
          params: {
            search: appliedSearch || undefined,
            status: statusFilter === 'all' ? undefined : statusFilter,
            page: pageToLoad,
            pageSize: pageSizeValue,
            sortBy,
            sortOrder,
          },
        });
        setPlayers(res.data.players);
        setStats(res.data.stats);
        setTotal(res.data.total);
        setPage(res.data.page);
        setPageSize(res.data.pageSize);
        setSelectedPlayer(prev =>
          prev && res.data.players.some(player => player.id === prev.id)
            ? prev
            : null
        );
        setError('');
      } catch {
        setError('Failed to load players');
      } finally {
        setLoading(false);
      }
    },
    [requestConfig, appliedSearch, statusFilter, sortBy, sortOrder]
  );

  useEffect(() => {
    loadPlayers(page, pageSize);
  }, [loadPlayers, page, pageSize]);

  const applyFilters = (e?: FormEvent) => {
    e?.preventDefault();
    setPage(1);
    setAppliedSearch(search.trim());
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  const handlePageChange = (nextPage: number) => {
    const safePage = Math.min(Math.max(nextPage, 1), totalPages);
    if (safePage !== page) {
      setPage(safePage);
    }
  };

  return (
    <div>
      <div className="section-header">
        <h2>Players</h2>
        <button onClick={() => loadPlayers(page, pageSize)} disabled={loading}>
          {loading ? 'Refreshing…' : 'Refresh'}
        </button>
      </div>
      <form className="filters" onSubmit={applyFilters}>
        <input
          placeholder="Search by ID, name or guild"
          value={search}
          onChange={e => setSearch(e.target.value)}
        />
        <div className="filters-row">
          <label>
            Status
            <select
              value={statusFilter}
              onChange={e => {
                setStatusFilter(e.target.value as typeof statusFilter);
                setPage(1);
              }}
            >
              <option value="all">All</option>
              <option value="online">Online</option>
              <option value="offline">Offline</option>
              <option value="banned">Banned</option>
            </select>
          </label>
          <label>
            Sort by
            <select
              value={sortBy}
              onChange={e => {
                setSortBy(e.target.value as typeof sortBy);
                setPage(1);
              }}
            >
              <option value="lastLogin">Last login</option>
              <option value="level">Level</option>
              <option value="name">Name</option>
              <option value="totalSpend">Total spend</option>
            </select>
          </label>
          <label>
            Order
            <select
              value={sortOrder}
              onChange={e => {
                setSortOrder(e.target.value as typeof sortOrder);
                setPage(1);
              }}
            >
              <option value="desc">Descending</option>
              <option value="asc">Ascending</option>
            </select>
          </label>
          <label>
            Page size
            <select
              value={pageSize}
              onChange={e => {
                const size = Number(e.target.value);
                setPageSize(size);
                setPage(1);
              }}
            >
              <option value={5}>5</option>
              <option value={10}>10</option>
              <option value={20}>20</option>
              <option value={50}>50</option>
            </select>
          </label>
        </div>
        <button type="submit">Apply filters</button>
      </form>
      {error && <p className="error">{error}</p>}
      {stats && (
        <>
          <div className="summary-grid">
            <div className="summary-card">
              <span>Overall Players</span>
              <strong>{stats.overall.total}</strong>
              <small>
                Online {stats.overall.online} · Banned {stats.overall.banned}
              </small>
            </div>
            <div className="summary-card">
              <span>Current Results</span>
              <strong>{stats.filtered.total}</strong>
              <small>
                Online {stats.filtered.online} · Avg level{' '}
                {stats.filtered.averageLevel}
              </small>
            </div>
          </div>
          <div className="chips">
            <span className="chip">High value overall {stats.overall.highValue}</span>
            <span className="chip">High value filtered {stats.filtered.highValue}</span>
            <span className="chip">Offline filtered {stats.filtered.offline}</span>
          </div>
          <div className="inline-stats">
            <span>
              Showing {players.length} of {stats.filtered.total} players
            </span>
            <span>Online: {stats.filtered.online}</span>
            <span>Offline: {stats.filtered.offline}</span>
            <span>Banned: {stats.filtered.banned}</span>
            <span>Avg level: {stats.filtered.averageLevel}</span>
          </div>
        </>
      )}
      <div className="table-wrapper">
        <table className="table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>Level</th>
              <th>VIP</th>
              <th>Status</th>
              <th>Total Spend</th>
              <th>Last Login</th>
              <th>Guild</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {players.length === 0 && (
              <tr>
                <td colSpan={8} style={{ textAlign: 'center' }}>
                  {loading ? 'Loading players…' : 'No players found'}
                </td>
              </tr>
            )}
            {players.map(player => (
              <tr
                key={player.id}
                className={selectedPlayer?.id === player.id ? 'active' : ''}
              >
                <td>{player.id}</td>
                <td>{player.name}</td>
                <td>{player.level}</td>
                <td>{player.vipLevel}</td>
                <td>
                  <span className={`status-tag status-${player.status}`}>
                    {player.status}
                  </span>
                </td>
                <td>
                  {typeof player.totalSpend === 'number'
                    ? `$${player.totalSpend.toFixed(2)}`
                    : '—'}
                </td>
                <td>{new Date(player.lastLogin).toLocaleString()}</td>
                <td>{player.guild || '—'}</td>
                <td className="table-actions">
                  <button
                    type="button"
                    className="button-secondary"
                    onClick={() => setSelectedPlayer(player)}
                  >
                    View
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="pagination">
        <button onClick={() => handlePageChange(page - 1)} disabled={page <= 1}>
          Previous
        </button>
        <span>
          Page {page} of {totalPages}
        </span>
        <button
          onClick={() => handlePageChange(page + 1)}
          disabled={page >= totalPages}
        >
          Next
        </button>
      </div>
      {selectedPlayer && (
        <div className="detail-card">
          <div className="section-header">
            <h3>Player detail</h3>
            <button
              type="button"
              className="button-secondary"
              onClick={() => setSelectedPlayer(null)}
            >
              Close
            </button>
          </div>
          <div className="chips">
            <span className="chip">Status: {selectedPlayer.status}</span>
            <span className="chip">VIP {selectedPlayer.vipLevel}</span>
            {selectedPlayer.region && (
              <span className="chip">Region: {selectedPlayer.region}</span>
            )}
          </div>
          <dl className="detail-grid">
            <div>
              <dt>Player ID</dt>
              <dd>{selectedPlayer.id}</dd>
            </div>
            <div>
              <dt>Name</dt>
              <dd>{selectedPlayer.name}</dd>
            </div>
            <div>
              <dt>Guild</dt>
              <dd>{selectedPlayer.guild || '—'}</dd>
            </div>
            <div>
              <dt>Level</dt>
              <dd>{selectedPlayer.level}</dd>
            </div>
            <div>
              <dt>Total spend</dt>
              <dd>
                {typeof selectedPlayer.totalSpend === 'number'
                  ? `$${selectedPlayer.totalSpend.toFixed(2)}`
                  : '—'}
              </dd>
            </div>
            <div>
              <dt>Last login</dt>
              <dd>{new Date(selectedPlayer.lastLogin).toLocaleString()}</dd>
            </div>
          </dl>
        </div>
      )}
    </div>
  );
}
