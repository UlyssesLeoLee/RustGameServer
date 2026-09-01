import { useEffect, useState } from 'react';
import axios from 'axios';

interface Props {
  auth: string;
}

interface MallItem {
  id: number;
  name: string;
  price: number;
}

export default function Mall({ auth }: Props) {
  const [items, setItems] = useState<MallItem[]>([]);
  const [name, setName] = useState('');
  const [price, setPrice] = useState('');
  const [status, setStatus] = useState('');
  const [statusLevel, setStatusLevel] = useState<'success' | 'error' | 'warning' | ''>('');
  const [search, setSearch] = useState('');
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editName, setEditName] = useState('');
  const [editPrice, setEditPrice] = useState('');
  const [saving, setSaving] = useState(false);

  const fetchItems = async () => {
    try {
      const res = await axios.get('/gm/mall/items', {
        headers: { Authorization: `Bearer ${auth}` }
      });
      setItems(res.data.items || []);
      setStatus('');
      setStatusLevel('');
    } catch {
      setStatus('Failed to fetch items');
      setStatusLevel('error');
    }
  };

  useEffect(() => {
    fetchItems();
  }, []);

  const createItem = async () => {
    if (!name.trim()) {
      setStatus('Name is required');
      setStatusLevel('warning');
      return;
    }
    const numericPrice = Number(price);
    if (!Number.isFinite(numericPrice)) {
      setStatus('Enter a valid price');
      setStatusLevel('warning');
      return;
    }
    try {
      setSaving(true);
      await axios.post(
        '/gm/mall/items',
        { name, price: numericPrice },
        { headers: { Authorization: `Bearer ${auth}` } }
      );
      setName('');
      setPrice('');
      setStatus('Item created');
      setStatusLevel('success');
      await fetchItems();
    } catch {
      setStatus('Failed to create item');
      setStatusLevel('error');
    } finally {
      setSaving(false);
    }
  };

  const deleteItem = async (id: number) => {
    try {
      setSaving(true);
      await axios.delete(`/gm/mall/items/${id}`, {
        headers: { Authorization: `Bearer ${auth}` }
      });
      setStatus('Item deleted');
      setStatusLevel('success');
      await fetchItems();
    } catch {
      setStatus('Failed to delete item');
      setStatusLevel('error');
    } finally {
      setSaving(false);
    }
  };

  const beginEdit = (item: MallItem) => {
    setEditingId(item.id);
    setEditName(item.name);
    setEditPrice(item.price.toString());
  };

  const cancelEdit = () => {
    setEditingId(null);
    setEditName('');
    setEditPrice('');
  };

  const saveEdit = async (id: number) => {
    const trimmedName = editName.trim();
    const numericPrice = Number(editPrice);
    if (!trimmedName) {
      setStatus('Name is required');
      setStatusLevel('warning');
      return;
    }
    if (!Number.isFinite(numericPrice)) {
      setStatus('Enter a valid price');
      setStatusLevel('warning');
      return;
    }
    try {
      setSaving(true);
      await axios.put(
        `/gm/mall/items/${id}`,
        { name: trimmedName, price: numericPrice },
        { headers: { Authorization: `Bearer ${auth}` } }
      );
      setStatus('Item updated');
      setStatusLevel('success');
      cancelEdit();
      await fetchItems();
    } catch {
      setStatus('Failed to update item');
      setStatusLevel('error');
    } finally {
      setSaving(false);
    }
  };

  const filteredItems = items.filter(item => {
    const query = search.trim().toLowerCase();
    if (!query) return true;
    return (
      item.name.toLowerCase().includes(query) ||
      item.id.toString().includes(query)
    );
  });

  const totalItems = filteredItems.length;
  const totalValue = filteredItems.reduce((acc, item) => acc + (item.price || 0), 0);
  const averagePrice = totalItems ? totalValue / totalItems : 0;

  const statusClass = statusLevel
    ? `status-message status-${statusLevel}`
    : 'status-message';

  return (
    <div>
      <section className="section">
        <div className="section-header">
          <h2>Mall Items</h2>
          <button onClick={fetchItems} disabled={saving}>
            Refresh
          </button>
        </div>
        <p className="section-description">
          Manage premium shop inventory, adjust pricing and ensure the latest
          catalog is available to players.
        </p>
        <div className="form-grid" style={{ marginBottom: '16px' }}>
          <div className="field">
            <label htmlFor="mall-search">Search</label>
            <input
              id="mall-search"
              placeholder="Search by ID or name"
              value={search}
              onChange={e => setSearch(e.target.value)}
            />
          </div>
          <div className="field">
            <label>Total items</label>
            <input value={filteredItems.length} readOnly />
          </div>
          <div className="field">
            <label>Average price</label>
            <input value={`$${averagePrice.toFixed(2)}`} readOnly />
          </div>
        </div>
        <div className="chips">
          <span className="chip">Catalog size {totalItems}</span>
          <span className="chip">Total value ${totalValue.toFixed(2)}</span>
        </div>
        <div className="table-wrapper">
          <table className="table">
            <thead>
              <tr>
                <th>ID</th>
                <th>Name</th>
                <th>Price</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredItems.length === 0 && (
                <tr>
                  <td colSpan={4} className="empty-state">
                    {items.length === 0
                      ? 'No items in the catalog yet'
                      : 'No items match the current search'}
                  </td>
                </tr>
              )}
              {filteredItems.map(item => {
                const isEditing = editingId === item.id;
                return (
                  <tr key={item.id}>
                    <td>{item.id}</td>
                    <td>
                      {isEditing ? (
                        <input
                          value={editName}
                          onChange={e => setEditName(e.target.value)}
                        />
                      ) : (
                        item.name
                      )}
                    </td>
                    <td>
                      {isEditing ? (
                        <input
                          value={editPrice}
                          onChange={e => setEditPrice(e.target.value)}
                        />
                      ) : (
                        `$${item.price.toFixed(2)}`
                      )}
                    </td>
                    <td className="table-actions">
                      {isEditing ? (
                        <>
                          <button
                            onClick={() => saveEdit(item.id)}
                            disabled={saving}
                          >
                            Save
                          </button>
                          <button
                            className="button-secondary"
                            onClick={cancelEdit}
                            disabled={saving}
                          >
                            Cancel
                          </button>
                        </>
                      ) : (
                        <>
                          <button
                            className="button-secondary"
                            onClick={() => beginEdit(item)}
                          >
                            Edit
                          </button>
                          <button
                            onClick={() => deleteItem(item.id)}
                            disabled={saving}
                          >
                            Delete
                          </button>
                        </>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </section>
      <section className="section">
        <h3>Add Item</h3>
        <div className="form-grid">
          <div className="field">
            <label htmlFor="mall-name">Name</label>
            <input
              id="mall-name"
              placeholder="Name"
              value={name}
              onChange={e => setName(e.target.value)}
            />
          </div>
          <div className="field">
            <label htmlFor="mall-price">Price</label>
            <input
              id="mall-price"
              placeholder="Price"
              value={price}
              onChange={e => setPrice(e.target.value)}
            />
          </div>
        </div>
        <button onClick={createItem} disabled={saving}>
          {saving ? 'Processing…' : 'Create'}
        </button>
      </section>
      {status && <p className={statusClass}>{status}</p>}
    </div>
  );
}

