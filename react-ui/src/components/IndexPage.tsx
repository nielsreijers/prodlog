import React, { useState, useEffect, useRef } from 'react';
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
  onSearchResults: (entries: LogEntrySummary[]) => void;
}

function FilterForm({ filters, onFiltersChange, onSearchResults }: FilterFormProps) {
  const navigate = useNavigate();
  
  // Local state for input values to prevent focus loss
  const [localHost, setLocalHost] = useState(filters.host || '');
  const [localSearch, setLocalSearch] = useState(filters.search || '');
  const [localDate, setLocalDate] = useState(filters.date || '');
  
  // Use ref to track current filters without causing re-renders
  const filtersRef = useRef(filters);
  filtersRef.current = filters;
  
  // Update local state when filters change from outside (e.g., URL changes)
  useEffect(() => {
    setLocalHost(filters.host || '');
    setLocalSearch(filters.search || '');
    setLocalDate(filters.date || '');
  }, [filters.host, filters.search, filters.date]);
  
  // Helper function to validate yyyy-mm-dd format
  const formatDateForAPI = (dateStr: string): string | undefined => {
    if (!dateStr) return undefined;
    
    // Accept yyyy-mm-dd format
    if (/^\d{4}-\d{1,2}-\d{1,2}$/.test(dateStr)) {
      const [year, month, day] = dateStr.split('-');
      return `${year}-${month.padStart(2, '0')}-${day.padStart(2, '0')}`;
    }
    
    return undefined;
  };
  
  // Debounced effect to search without updating URL
  useEffect(() => {
    // Check if local values differ from current URL filters
    const currentFilters = filtersRef.current;
    const localFilters = {
      date: formatDateForAPI(localDate),
      host: localHost || undefined,
      search: localSearch || undefined
    };
    
    // Only search if local values are different from URL
    const hasChanged = 
      localFilters.date !== currentFilters.date ||
      localFilters.host !== currentFilters.host ||
      localFilters.search !== currentFilters.search;
    
    if (!hasChanged) return;
    
    const timeoutId = setTimeout(() => {
      // Call API directly without updating URL
      const searchFilters = {
        ...localFilters,
        show_noop: currentFilters.show_noop,
      };
      
      // Trigger search without URL update
      const loadEntries = async () => {
        try {
          const data = await api.getEntriesSummary(searchFilters);
          if (onSearchResults) {
            onSearchResults(data);
          }
        } catch (err) {
          console.error('Search failed:', err);
        }
      };
      
      loadEntries();
    }, 300); // 300ms delay
    
    return () => clearTimeout(timeoutId);
  }, [localHost, localSearch, localDate]);
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Force immediate update on form submit
    onFiltersChange({
      date: formatDateForAPI(localDate),
      show_noop: filtersRef.current.show_noop,
      host: localHost || undefined,
      search: localSearch || undefined
    });
  };

  const handleNoopToggle = (checked: boolean) => {
    onFiltersChange({ ...filters, show_noop: checked });
  };

  const clearFilters = () => {
    setLocalHost('');
    setLocalSearch('');
    setLocalDate('');
    onFiltersChange({});
  };

  return (
    <div className="filters">
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          placeholder="Date (yyyy-mm-dd)"
          value={localDate}
          onChange={(e) => setLocalDate(e.target.value)}
        />
        <input
          type="text"
          placeholder="Hostname"
          value={localHost}
          onChange={(e) => setLocalHost(e.target.value)}
        />
        <input
          type="text"
          placeholder="Command or message"
          value={localSearch}
          onChange={(e) => setLocalSearch(e.target.value)}
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

  // Load entries when URL changes (bookmarks, back/forward navigation)
  useEffect(() => {
    const loadEntries = async () => {
      try {
        setLoading(true);
        const data = await api.getEntriesSummary(filters);
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
      
      <FilterForm filters={filters} onFiltersChange={updateFilters} onSearchResults={setEntries} />
      
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