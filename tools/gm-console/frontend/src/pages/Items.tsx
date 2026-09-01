import { useCallback, useEffect, useMemo, useState } from 'react';
import axios from 'axios';

interface Props {
  auth: string;
}

export default function Items({ auth }: Props) {
  const [playerId, setPlayerId] = useState('');
  const [itemId, setItemId] = useState('');
  const [amount, setAmount] = useState('1');
  const [status, setStatus] = useState('');
  const [statusLevel, setStatusLevel] = useState<'success' | 'error' | 'warning' | ''>('');
  const [grants, setGrants] = useState<GrantRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [historySearch, setHistorySearch] = useState('');
  const [historyLimit, setHistoryLimit] = useState(20);

  interface GrantRecord {
    id: number;
    playerId: string;
    itemId: string;
    amount: number;
    admin?: string;
    createdAt?: string;
  }

  const requestConfig = useMemo(
    () => ({ headers: { Authorization: `Bearer ${auth}` } }),
    [auth]
  );

  const fetchGrants = useCallback(async () => {
    try {
      const res = await axios.get<{ grants: GrantRecord[] }>(
        '/gm/items/grants',
        requestConfig
      );
      setGrants(res.data.grants || []);
      setError('');
    } catch {
      setError('Failed to load grant history');
    }
  }, [requestConfig]);

  useEffect(() => {
    fetchGrants();
  }, [fetchGrants]);

  const grant = async () => {
    if (!playerId.trim() || !itemId.trim()) {
      setStatus('Player ID and Item ID are required');
      setStatusLevel('warning');
      return;
    }
    try {
      setLoading(true);
      const res = await axios.post(
        '/gm/items/grant',
        { player_id: playerId, item_id: itemId, amount: Number(amount) },
        requestConfig
      );
      setStatus(res.data.status || 'granted');
      setStatusLevel('success');
      await fetchGrants();
    } catch {
      setStatus('Failed to grant');
      setStatusLevel('error');
    } finally {
      setLoading(false);
    }
  };

  const filteredGrants = useMemo(() => {
    const query = historySearch.trim().toLowerCase();
    const matches = grants.filter(grant => {
      if (!query) return true;
      const textPool = [
        grant.playerId,
        grant.itemId,
        grant.admin ?? '',
        grant.amount?.toString() ?? '',
      ];
      return textPool.some(value => value.toLowerCase().includes(query));
    });
    return matches.slice(0, historyLimit);
  }, [grants, historySearch, historyLimit]);

  const totalGranted = useMemo(
    () => filteredGrants.reduce((acc, grant) => acc + (grant.amount || 0), 0),
    [filteredGrants]
  );

  const statusClass = statusLevel
    ? `status-message status-${statusLevel}`
    : 'status-message';

  return (
    <div>
      <section className="section">
        <div className="section-header">
          <h2>Item & Activity Management</h2>
          <button
            className="button-secondary"
            onClick={() => {
              setPlayerId('');
              setItemId('');
              setAmount('1');
              setStatus('');
              setStatusLevel('');
            }}
          >
            Reset form
          </button>
        </div>
        <p className="section-description">
          Grant virtual items directly to players and keep track of the operation
          audit trail.
        </p>
        <div className="form-grid">
          <div className="field">
            <label htmlFor="grant-player">Player ID</label>
            <input
              id="grant-player"
              placeholder="Player ID"
              value={playerId}
              onChange={e => setPlayerId(e.target.value)}
            />
          </div>
          <div className="field">
            <label htmlFor="grant-item">Item ID</label>
            <input
              id="grant-item"
              placeholder="Item ID"
              value={itemId}
              onChange={e => setItemId(e.target.value)}
            />
          </div>
          <div className="field">
            <label htmlFor="grant-amount">Amount</label>
            <input
              id="grant-amount"
              placeholder="Amount"
              value={amount}
              onChange={e => setAmount(e.target.value)}
            />
          </div>
        </div>
        <button onClick={grant} disabled={loading}>
          {loading ? 'Granting…' : 'Grant Item'}
        </button>
        {status && <p className={statusClass}>{status}</p>}
      </section>
      <section className="section">
        <div className="section-header">
          <h3>Recent Grants</h3>
          <button className="button-secondary" onClick={fetchGrants}>
            Refresh history
          </button>
        </div>
        <p className="section-description">
          Review the most recent grants issued by the GM team. Use the quick
          filters to locate individual operations.
        </p>
        <div className="form-grid" style={{ marginBottom: '16px' }}>
          <div className="field">
            <label htmlFor="grant-search">Filter</label>
            <input
              id="grant-search"
              placeholder="Filter by player, item or admin"
              value={historySearch}
              onChange={e => setHistorySearch(e.target.value)}
            />
          </div>
          <div className="field">
            <label htmlFor="grant-limit">Show</label>
            <select
              id="grant-limit"
              value={historyLimit}
              onChange={e => setHistoryLimit(Number(e.target.value))}
            >
              <option value={10}>10 records</option>
              <option value={20}>20 records</option>
              <option value={50}>50 records</option>
            </select>
          </div>
        </div>
        <div className="chips">
          <span className="chip">Entries shown {filteredGrants.length}</span>
          <span className="chip">Total amount {totalGranted}</span>
        </div>
        {error && <p className="error">{error}</p>}
        <div className="table-wrapper">
          <table className="table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Player</th>
                <th>Item</th>
                <th>Amount</th>
                <th>Operator</th>
                <th>Created At</th>
              </tr>
            </thead>
            <tbody>
              {filteredGrants.length === 0 && (
                <tr>
                  <td colSpan={6} className="empty-state">
                    {grants.length === 0
                      ? 'No grants recorded yet'
                      : 'No grants matching the filter'}
                  </td>
                </tr>
              )}
              {filteredGrants.map(grant => (
                <tr key={grant.id}>
                  <td>{grant.id}</td>
                  <td>{grant.playerId}</td>
                  <td>{grant.itemId}</td>
                  <td>{grant.amount}</td>
                  <td>{grant.admin || '—'}</td>
                  <td>
                    {grant.createdAt
                      ? new Date(grant.createdAt).toLocaleString()
                      : '—'}
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
