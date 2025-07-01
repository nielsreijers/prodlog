import React, { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { LogEntrySummary, Filters } from '../types';
import { api } from '../api';

const CopyIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M8 4v12a2 2 0 002 2h8a2 2 0 002-2V7.242a2 2 0 00-.602-1.43L16.083 2.57A2 2 0 0014.685 2H10a2 2 0 00-2 2z"/>
    <path d="M16 18v2a2 2 0 01-2 2H6a2 2 0 01-2-2V9a2 2 0 012-2h2"/>
  </svg>
);

const CheckIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M20 6L9 17l-5-5"/>
  </svg>
);

const RunIcon = () => (
  <svg fill="none" stroke="currentColor" strokeWidth="1" width="24" height="24">
    <path d="M4 6l4 4-4 4" strokeLinecap="round"/>
    <path d="M12 14h6" strokeLinecap="round"/>
  </svg>
);

const EditIcon = () => (
  <svg fill="none" stroke="currentColor" strokeWidth="1" width="16" height="16">
    <path d="M8 1h4a2 2 0 012 2v10a2 2 0 01-2 2H4a2 2 0 01-2-2V3a2 2 0 012-2z"/>
    <path d="M4 4h7 M4 7h7 M4 10h5"/>
  </svg>
);

interface CopyButtonProps {
  entry: LogEntrySummary;
}

function CopyButton({ entry }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);
  
  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent row click navigation
    
    try {
      const copyText = api.getCopyText(entry);
      await api.copyToClipboard(copyText);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  return (
    <button 
      className={`edit-or-copy-button ${copied ? 'copied' : ''}`}
      onClick={handleCopy}
      title="Copy"
    >
      {copied ? <CheckIcon /> : <CopyIcon />}
    </button>
  );
}

interface EntryRowProps {
  entry: LogEntrySummary;
  onClick: () => void;
}

function EntryRow({ entry, onClick }: EntryRowProps) {
  const rowClass = entry.is_noop ? 'noop-row' : 
                  entry.exit_code !== 0 ? 'error-row' : '';
  
  const EntryTypeIcon = entry.capture_type === 'Run' ? RunIcon : EditIcon;
  
  const messageRow = entry.message ? (
    <tr className="message-row clickable-row" onClick={onClick}>
      <td colSpan={2}></td>
      <td colSpan={5} className="message-row">
        <div>
          <span>{entry.message}</span>
        </div>
      </td>
    </tr>
  ) : null;

  return (
    <tbody>
      <tr className={`main-row clickable-row ${rowClass}`} onClick={onClick}>
        <td><EntryTypeIcon /></td>
        <td>{api.formatTimestamp(entry.start_time)}</td>
        <td>{entry.host}</td>
        <td>{entry.cmd}</td>
        <td>
          <div className="button-group">
            <CopyButton entry={entry} />
          </div>
        </td>
        <td>{entry.duration_ms}ms</td>
        <td>{entry.exit_code}</td>
      </tr>
      {messageRow}
    </tbody>
  );
}

interface FilterFormProps {
  filters: Filters;
  onFiltersChange: (filters: Filters) => void;
}

function FilterForm({ filters, onFiltersChange }: FilterFormProps) {
  const navigate = useNavigate();
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // URL will be updated by the effect in IndexPage
  };

  const handleNoopToggle = (checked: boolean) => {
    onFiltersChange({ ...filters, show_noop: checked });
  };

  const clearFilters = () => {
    onFiltersChange({});
  };

  return (
    <div className="filters">
      <form onSubmit={handleSubmit}>
        <input
          type="date"
          value={filters.date || ''}
          onChange={(e) => onFiltersChange({ ...filters, date: e.target.value || undefined })}
        />
        <input
          type="text"
          placeholder="Hostname"
          value={filters.host || ''}
          onChange={(e) => onFiltersChange({ ...filters, host: e.target.value || undefined })}
        />
        <input
          type="text"
          placeholder="Command or message"
          value={filters.search || ''}
          onChange={(e) => onFiltersChange({ ...filters, search: e.target.value || undefined })}
        />
        <label className="switch">
          <input
            type="checkbox"
            checked={filters.show_noop || false}
            onChange={(e) => handleNoopToggle(e.target.checked)}
          />
          <span className="slider"></span>
        </label>
        <span className="switch-label">Reveal no-op entries</span>
        <button className="bluebutton" type="submit">Filter</button>
        <button className="greybutton" type="button" onClick={clearFilters}>Clear</button>
      </form>
      <div className="filters-right">
        <button className="bluebutton" type="button" onClick={() => navigate('/redact')}>
          Redact Passwords
        </button>
      </div>
    </div>
  );
}

export default function IndexPage() {
  const [entries, setEntries] = useState<LogEntrySummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  
  // Parse filters from URL
  const filters: Filters = {
    date: searchParams.get('date') || undefined,
    host: searchParams.get('host') || undefined,
    search: searchParams.get('search') || undefined,
    show_noop: searchParams.get('show_noop') === 'true' || undefined,
  };

  // Update URL when filters change
  const updateFilters = (newFilters: Filters) => {
    const params = new URLSearchParams();
    if (newFilters.date) params.set('date', newFilters.date);
    if (newFilters.host) params.set('host', newFilters.host);
    if (newFilters.search) params.set('search', newFilters.search);
    if (newFilters.show_noop) params.set('show_noop', 'true');
    
    setSearchParams(params);
  };

  // Load entries
  useEffect(() => {
    const loadEntries = async () => {
      try {
        setLoading(true);
        const data = await api.getEntriesSummary(filters);
        // Sort entries by start time, newest first
        data.sort((a, b) => new Date(b.start_time).getTime() - new Date(a.start_time).getTime());
        setEntries(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load entries');
      } finally {
        setLoading(false);
      }
    };

    loadEntries();
  }, [searchParams]);

  const handleRowClick = (uuid: string) => {
    navigate(`/entry/${uuid}`);
  };

  if (loading) {
    return (
      <div className="container">
        <div className="header">
          <h1>Prodlog Viewer</h1>
        </div>
        <div>Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="container">
        <div className="header">
          <h1>Prodlog Viewer</h1>
        </div>
        <div className="message error">Error loading log data: {error}</div>
      </div>
    );
  }

  return (
    <div className="container">
      <div className="header">
        <h1>Prodlog Viewer</h1>
      </div>
      
      <FilterForm filters={filters} onFiltersChange={updateFilters} />
      
      <table>
        <thead>
          <tr>
            <th style={{width: '24px'}}></th>
            <th style={{width: '190px'}}>Time</th>
            <th style={{width: '160px'}}>Host</th>
            <th style={{width: 'auto', whiteSpace: 'normal'}}>Command</th>
            <th style={{width: '48px'}}></th>
            <th style={{width: '80px'}}>Duration</th>
            <th style={{width: '30px'}}>Exit</th>
          </tr>
        </thead>
        {entries.map(entry => (
          <EntryRow 
            key={entry.uuid} 
            entry={entry} 
            onClick={() => handleRowClick(entry.uuid)}
          />
        ))}
      </table>
    </div>
  );
} 