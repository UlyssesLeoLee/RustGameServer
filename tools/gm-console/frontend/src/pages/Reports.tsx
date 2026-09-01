import { useEffect, useMemo, useState } from 'react';
import axios from 'axios';

interface Props {
  auth: string;
}

type ReportRecord = {
  id?: number | string;
  type?: string;
  category?: string;
  period?: string;
  range?: string;
  window?: string;
  generatedAt?: string;
  summary?: string;
  metrics?: Record<string, number | string>;
  [key: string]: unknown;
};

const asString = (value: unknown): string => {
  if (typeof value === 'string') return value;
  if (typeof value === 'number') return value.toString();
  return '';
};

const getReportType = (record: ReportRecord): string =>
  asString(record.type) ||
  asString(record.category) ||
  asString(record['reportType']) ||
  'General';

const getReportPeriod = (record: ReportRecord): string =>
  asString(record.period) || asString(record.range) || asString(record.window) || '—';

const getTimestamp = (record: ReportRecord): string =>
  asString(record.generatedAt) ||
  asString(record['createdAt']) ||
  asString(record['updatedAt']) ||
  '';

const formatTimestamp = (value: string): string => {
  if (!value) return '—';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString();
};

export default function Reports({ auth }: Props) {
  const [reports, setReports] = useState<ReportRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [selectedType, setSelectedType] = useState('all');
  const [selectedPeriod, setSelectedPeriod] = useState('all');
  const [search, setSearch] = useState('');
  const [selectedReport, setSelectedReport] = useState<ReportRecord | null>(null);

  const fetchReports = async () => {
    setLoading(true);
    try {
      const res = await axios.get<{ reports: ReportRecord[] }>(
        '/gm/reports',
        {
          headers: { Authorization: `Bearer ${auth}` }
        }
      );
      const list = Array.isArray(res.data.reports)
        ? res.data.reports
        : [];
      setReports(list);
      setError('');
      setSelectedReport(list[0] || null);
    } catch {
      setError('Failed to fetch reports');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchReports();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const uniqueTypes = useMemo(() => {
    const types = new Set<string>();
    reports.forEach(report => {
      const type = getReportType(report);
      if (type) types.add(type);
    });
    return Array.from(types).sort();
  }, [reports]);

  const uniquePeriods = useMemo(() => {
    const periods = new Set<string>();
    reports.forEach(report => {
      const period = getReportPeriod(report);
      if (period && period !== '—') periods.add(period);
    });
    return Array.from(periods).sort();
  }, [reports]);

  const filteredReports = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return reports.filter(report => {
      if (selectedType !== 'all' && getReportType(report) !== selectedType) {
        return false;
      }
      if (selectedPeriod !== 'all' && getReportPeriod(report) !== selectedPeriod) {
        return false;
      }
      if (!needle) return true;
      const text = JSON.stringify(report).toLowerCase();
      return text.includes(needle);
    });
  }, [reports, selectedType, selectedPeriod, search]);

  useEffect(() => {
    setSelectedReport(prev => {
      if (!prev) return filteredReports[0] || null;
      return (
        filteredReports.find(report => report.id === prev.id) ||
        filteredReports[0] ||
        null
      );
    });
  }, [filteredReports]);

  const sortedReports = useMemo(() => {
    return filteredReports
      .slice()
      .sort(
        (a, b) =>
          (new Date(getTimestamp(b)).getTime() || 0) -
          (new Date(getTimestamp(a)).getTime() || 0)
      );
  }, [filteredReports]);

  const aggregatedMetrics = useMemo(() => {
    const totals = new Map<string, number>();
    sortedReports.forEach(report => {
      const metrics = report.metrics;
      if (metrics && typeof metrics === 'object') {
        Object.entries(metrics).forEach(([key, value]) => {
          const numeric = Number(value);
          if (Number.isFinite(numeric)) {
            totals.set(key, (totals.get(key) ?? 0) + numeric);
          }
        });
      }
    });
    return Array.from(totals.entries());
  }, [sortedReports]);

  return (
    <div>
      <section className="section">
        <div className="section-header">
          <h2>Reports</h2>
          <button onClick={fetchReports} disabled={loading}>
            {loading ? 'Loading…' : 'Refresh'}
          </button>
        </div>
        <p className="section-description">
          Analyse behaviour across retention, monetisation and performance
          reports. Filter by segment to compare trends.
        </p>
        <div className="form-grid">
          <div className="field">
            <label htmlFor="report-type">Report type</label>
            <select
              id="report-type"
              value={selectedType}
              onChange={e => setSelectedType(e.target.value)}
            >
              <option value="all">All types</option>
              {uniqueTypes.map(type => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <label htmlFor="report-period">Period</label>
            <select
              id="report-period"
              value={selectedPeriod}
              onChange={e => setSelectedPeriod(e.target.value)}
            >
              <option value="all">All periods</option>
              {uniquePeriods.map(period => (
                <option key={period} value={period}>
                  {period}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <label htmlFor="report-search">Search</label>
            <input
              id="report-search"
              placeholder="Search in titles and metrics"
              value={search}
              onChange={e => setSearch(e.target.value)}
            />
          </div>
        </div>
        {error && <p className="error">{error}</p>}
        {aggregatedMetrics.length > 0 && (
          <div className="summary-grid">
            {aggregatedMetrics.map(([key, value]) => (
              <div className="summary-card" key={key}>
                <span>{key}</span>
                <strong>{value}</strong>
                <small>Combined across filtered reports</small>
              </div>
            ))}
          </div>
        )}
        <div className="table-wrapper">
          <table className="table">
            <thead>
              <tr>
                <th>Report</th>
                <th>Period</th>
                <th>Highlights</th>
                <th>Generated</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {sortedReports.length === 0 && (
                <tr>
                  <td colSpan={5} className="empty-state">
                    {reports.length === 0
                      ? 'No reports available yet'
                      : 'No reports match the current filters'}
                  </td>
                </tr>
              )}
              {sortedReports.map((report, index) => {
                const metrics =
                  report.metrics && typeof report.metrics === 'object'
                    ? Object.entries(report.metrics)
                    : [];
                const preview = metrics
                  .slice(0, 3)
                  .map(([key, value]) => `${key}: ${value}`)
                  .join(' · ');
                const reportId = asString(report.id) || String(index + 1);
                const isActive =
                  selectedReport === report ||
                  (selectedReport && selectedReport.id === report.id);
                return (
                  <tr
                    key={`${reportId}-${getReportType(report)}-${getReportPeriod(report)}`}
                    className={isActive ? 'active' : ''}
                  >
                    <td>
                      <div className="list-title">{getReportType(report)}</div>
                      <div className="muted">#{reportId}</div>
                    </td>
                    <td>{getReportPeriod(report)}</td>
                    <td>{report.summary || preview || '—'}</td>
                    <td>{formatTimestamp(getTimestamp(report))}</td>
                    <td className="table-actions">
                      <button
                        type="button"
                        className="button-secondary"
                        onClick={() => setSelectedReport(report)}
                      >
                        View
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        {sortedReports.length > 0 && (
          <p className="muted">
            Showing {sortedReports.length} of {reports.length} reports
          </p>
        )}
      </section>
      {selectedReport && (
        <div className="detail-card">
          <div className="section-header">
            <h3>Report detail</h3>
            <button
              type="button"
              className="button-secondary"
              onClick={() => setSelectedReport(null)}
            >
              Close
            </button>
          </div>
          <div className="chips">
            <span className="chip">Type: {getReportType(selectedReport)}</span>
            <span className="chip">Period: {getReportPeriod(selectedReport)}</span>
            <span className="chip">
              Generated: {formatTimestamp(getTimestamp(selectedReport))}
            </span>
          </div>
          <pre className="players">
            {JSON.stringify(selectedReport, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}
