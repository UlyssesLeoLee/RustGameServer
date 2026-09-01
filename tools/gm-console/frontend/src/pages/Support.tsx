import { useCallback, useEffect, useMemo, useState } from 'react';
import axios from 'axios';

interface Props {
  auth: string;
}

export default function Support({ auth }: Props) {
  const [playerId, setPlayerId] = useState('');
  const [message, setMessage] = useState('');
  const [status, setStatus] = useState('');
  const [statusLevel, setStatusLevel] = useState<'success' | 'error' | 'warning' | ''>('');
  const [tickets, setTickets] = useState<Ticket[]>([]);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState('');
  const [filterStatus, setFilterStatus] = useState<
    'all' | 'open' | 'pending' | 'resolved'
  >('all');
  const [selectedTicket, setSelectedTicket] = useState<Ticket | null>(null);

  interface Ticket {
    id: number;
    playerId: string;
    message: string;
    status: 'open' | 'pending' | 'resolved';
    admin?: string;
    createdAt?: string;
    updatedAt?: string;
  }

  const requestConfig = useMemo(
    () => ({ headers: { Authorization: `Bearer ${auth}` } }),
    [auth]
  );

  const fetchTickets = useCallback(async () => {
    try {
      const res = await axios.get<{ tickets: Ticket[] }>(
        '/gm/support/tickets',
        requestConfig
      );
      setTickets(res.data.tickets || []);
      setError('');
    } catch {
      setError('Failed to load tickets');
    }
  }, [requestConfig]);

  useEffect(() => {
    fetchTickets();
  }, [fetchTickets]);

  useEffect(() => {
    setSelectedTicket(prev => {
      if (!prev) return tickets[0] || null;
      return tickets.find(ticket => ticket.id === prev.id) || tickets[0] || null;
    });
  }, [tickets]);

  const send = async () => {
    if (!playerId.trim() || !message.trim()) {
      setStatus('Player ID and message are required');
      setStatusLevel('warning');
      return;
    }
    try {
      setLoading(true);
      const res = await axios.post(
        '/gm/support',
        { player_id: playerId, message },
        requestConfig
      );
      setStatus(res.data.status || 'sent');
      setStatusLevel('success');
      setPlayerId('');
      setMessage('');
      await fetchTickets();
    } catch {
      setStatus('Failed to send');
      setStatusLevel('error');
    } finally {
      setLoading(false);
    }
  };

  const updateTicketStatus = async (id: number, next: Ticket['status']) => {
    try {
      await axios.patch(
        `/gm/support/tickets/${id}`,
        { status: next },
        requestConfig
      );
      setStatus(`Ticket ${id} updated to ${next}`);
      setStatusLevel('success');
      fetchTickets();
    } catch {
      setError('Failed to update ticket status');
    }
  };

  const filteredTickets = useMemo(() => {
    const query = search.trim().toLowerCase();
    return tickets.filter(ticket => {
      if (filterStatus !== 'all' && ticket.status !== filterStatus) {
        return false;
      }
      if (!query) return true;
      const text = [
        ticket.playerId,
        ticket.message,
        ticket.admin ?? '',
        ticket.id.toString(),
      ]
        .join(' ')
        .toLowerCase();
      return text.includes(query);
    });
  }, [tickets, search, filterStatus]);

  useEffect(() => {
    setSelectedTicket(prev => {
      if (!prev) return filteredTickets[0] || null;
      return (
        filteredTickets.find(ticket => ticket.id === prev.id) ||
        filteredTickets[0] ||
        null
      );
    });
  }, [filteredTickets]);

  const ticketStats = useMemo(() => {
    return filteredTickets.reduce(
      (acc, ticket) => {
        acc.total += 1;
        acc[ticket.status] += 1;
        return acc;
      },
      { total: 0, open: 0, pending: 0, resolved: 0 }
    );
  }, [filteredTickets]);

  const statusClass = statusLevel
    ? `status-message status-${statusLevel}`
    : 'status-message';

  return (
    <div>
      <section className="section">
        <h2>Customer Support</h2>
        <p className="section-description">
          Log player-facing issues and keep track of ticket progress to ensure
          timely responses.
        </p>
        <div className="form-grid">
          <div className="field">
            <label htmlFor="ticket-player">Player ID</label>
            <input
              id="ticket-player"
              placeholder="Player ID"
              value={playerId}
              onChange={e => setPlayerId(e.target.value)}
            />
          </div>
          <div className="field" style={{ gridColumn: '1 / -1' }}>
            <label htmlFor="ticket-message">Message</label>
            <textarea
              id="ticket-message"
              placeholder="Describe the issue"
              value={message}
              onChange={e => setMessage(e.target.value)}
              rows={4}
            />
          </div>
        </div>
        <button onClick={send} disabled={loading}>
          {loading ? 'Submitting…' : 'Send'}
        </button>
        {status && <p className={statusClass}>{status}</p>}
      </section>
      <section className="section">
        <div className="section-header">
          <h3>Tickets</h3>
          <button className="button-secondary" onClick={fetchTickets}>
            Refresh
          </button>
        </div>
        <div className="form-grid" style={{ marginBottom: '16px' }}>
          <div className="field">
            <label htmlFor="ticket-search">Search</label>
            <input
              id="ticket-search"
              placeholder="Search by ID, player or message"
              value={search}
              onChange={e => setSearch(e.target.value)}
            />
          </div>
          <div className="field">
            <label htmlFor="ticket-status">Status</label>
            <select
              id="ticket-status"
              value={filterStatus}
              onChange={e =>
                setFilterStatus(e.target.value as typeof filterStatus)
              }
            >
              <option value="all">All</option>
              <option value="open">Open</option>
              <option value="pending">Pending</option>
              <option value="resolved">Resolved</option>
            </select>
          </div>
        </div>
        <div className="chips">
          <span className="chip">Total {ticketStats.total}</span>
          <span className="chip">Open {ticketStats.open}</span>
          <span className="chip">Pending {ticketStats.pending}</span>
          <span className="chip">Resolved {ticketStats.resolved}</span>
        </div>
        {error && <p className="error">{error}</p>}
        <div className="table-wrapper">
          <table className="table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Player</th>
                <th>Message</th>
                <th>Status</th>
                <th>Created</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredTickets.length === 0 && (
                <tr>
                  <td colSpan={6} className="empty-state">
                    {tickets.length === 0
                      ? 'No tickets yet'
                      : 'No tickets match the filter'}
                  </td>
                </tr>
              )}
              {filteredTickets.map(ticket => (
                <tr
                  key={ticket.id}
                  className={selectedTicket?.id === ticket.id ? 'active' : ''}
                >
                  <td>{ticket.id}</td>
                  <td>{ticket.playerId}</td>
                  <td>{ticket.message}</td>
                  <td>
                    <span className={`status-tag status-${ticket.status}`}>
                      {ticket.status}
                    </span>
                  </td>
                  <td>
                    {ticket.createdAt
                      ? new Date(ticket.createdAt).toLocaleString()
                      : '—'}
                  </td>
                  <td className="table-actions">
                    <button
                      type="button"
                      className="button-secondary"
                      onClick={() => setSelectedTicket(ticket)}
                    >
                      View
                    </button>
                    {ticket.status !== 'pending' && (
                      <button
                        type="button"
                        className="button-secondary"
                        onClick={() => updateTicketStatus(ticket.id, 'pending')}
                      >
                        Mark pending
                      </button>
                    )}
                    {ticket.status === 'resolved' ? (
                      <button
                        type="button"
                        onClick={() => updateTicketStatus(ticket.id, 'open')}
                      >
                        Reopen
                      </button>
                    ) : (
                      <button
                        type="button"
                        onClick={() => updateTicketStatus(ticket.id, 'resolved')}
                      >
                        Resolve
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
      {selectedTicket && (
        <div className="detail-card">
          <div className="section-header">
            <h3>Ticket detail</h3>
            <button
              type="button"
              className="button-secondary"
              onClick={() => setSelectedTicket(null)}
            >
              Close
            </button>
          </div>
          <div className="chips">
            <span className="chip">Status: {selectedTicket.status}</span>
            <span className="chip">Player: {selectedTicket.playerId}</span>
            <span className="chip">Admin: {selectedTicket.admin || '—'}</span>
          </div>
          <dl className="detail-grid">
            <div>
              <dt>Ticket ID</dt>
              <dd>{selectedTicket.id}</dd>
            </div>
            <div>
              <dt>Created</dt>
              <dd>
                {selectedTicket.createdAt
                  ? new Date(selectedTicket.createdAt).toLocaleString()
                  : '—'}
              </dd>
            </div>
            <div>
              <dt>Updated</dt>
              <dd>
                {selectedTicket.updatedAt
                  ? new Date(selectedTicket.updatedAt).toLocaleString()
                  : '—'}
              </dd>
            </div>
          </dl>
          <p style={{ whiteSpace: 'pre-wrap', marginTop: 8 }}>{selectedTicket.message}</p>
        </div>
      )}
    </div>
  );
}
