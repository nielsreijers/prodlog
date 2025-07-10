import React, { useState, useEffect, useRef } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { LogEntrySummary, Filters, Task } from '../types';
import { api } from '../api';
import DateRangePicker from './DateRangePicker';
import { TaskManager } from './TaskManager';
import { TaskGroup as TaskGroupComponent } from './TaskGroup';

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

const ExpandedIcon = () => (
  <svg fill="none" stroke="currentColor" width="16" height="16">
    <path stroke-width="2" d="M2 8h12"/>
  </svg>
);

const CollapsedIcon = () => (
  <svg fill="none" stroke="currentColor" width="16" height="16">
    <path stroke-width="2" d="M8 2v12"/>
    <path stroke-width="2" d="M2 8h12"/>
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
  isSelectMode?: boolean;
  isSelected?: boolean;
  onSelectionChange?: (uuid: string, isSelected: boolean) => void;
}

interface FilterFormProps {
  filters: Filters;
  onFiltersChange: (filters: Filters) => void;
  onSearchResults: (entries: LogEntrySummary[]) => void;
  onExpandToggle: () => void;
  allExpanded: boolean;
}

function FilterForm({ filters, onFiltersChange, onSearchResults, onExpandToggle, allExpanded }: FilterFormProps) {
  const navigate = useNavigate();
  
  // Local state for input values to prevent focus loss
  const [localHost, setLocalHost] = useState(filters.host || '');
  const [localSearch, setLocalSearch] = useState(filters.search || '');
  const [localSearchContent, setLocalSearchContent] = useState(filters.search_content || '');
  const [localDateRange, setLocalDateRange] = useState({
    from: filters.date_from || '',
    to: filters.date_to || ''
  });
  
  // Use ref to track current filters without causing re-renders
  const filtersRef = useRef(filters);
  filtersRef.current = filters;
  
  // Update local state when filters change from outside (e.g., URL changes)
  useEffect(() => {
    setLocalHost(filters.host || '');
    setLocalSearch(filters.search || '');
    setLocalSearchContent(filters.search_content || '');
    setLocalDateRange({
      from: filters.date_from || '',
      to: filters.date_to || ''
    });
  }, [filters.host, filters.search, filters.search_content, filters.date_from, filters.date_to]);
  
  // Debounced effect to search without updating URL
  useEffect(() => {
    // Check if local values differ from current URL filters
    const currentFilters = filtersRef.current;
    
    // Only search if local values are different from URL
    const hasChanged = 
      localDateRange.from !== currentFilters.date_from ||
      localDateRange.to !== currentFilters.date_to ||
      localHost !== currentFilters.host ||
      localSearch !== currentFilters.search ||
      localSearchContent !== currentFilters.search_content;
    
    if (!hasChanged) return;
    
    const timeoutId = setTimeout(() => {
      // Call API directly without updating URL
      const searchFilters = {
        date_from: localDateRange.from || undefined,
        date_to: localDateRange.to || undefined,
        host: localHost || undefined,
        search: localSearch || undefined,
        search_content: localSearchContent || undefined,
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
  }, [localHost, localSearch, localSearchContent, localDateRange.from, localDateRange.to]);
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Force immediate update on form submit
    onFiltersChange({
      date_from: localDateRange.from || undefined,
      date_to: localDateRange.to || undefined,
      show_noop: filtersRef.current.show_noop,
      host: localHost || undefined,
      search: localSearch || undefined,
      search_content: localSearchContent || undefined
    });
  };

  const handleNoopToggle = (checked: boolean) => {
    onFiltersChange({ ...filters, show_noop: checked });
  };

  const clearFilters = () => {
    setLocalHost('');
    setLocalSearch('');
    setLocalSearchContent('');
    setLocalDateRange({ from: '', to: '' });
    onFiltersChange({});
  };

  return (
    <div className="filters">
      <form onSubmit={handleSubmit}>
        <button className="greybutton  filters-right" onClick={onExpandToggle}>
          {allExpanded ? 'Collapse All' : 'Expand All'}
        </button>
        <DateRangePicker
          value={localDateRange}
          onChange={(range) => setLocalDateRange(range)}
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
        <input
          type="text"
          placeholder="Search all content (slow)"
          value={localSearchContent}
          onChange={(e) => setLocalSearchContent(e.target.value)}
        />
        <label className="switch">
          <input
            type="checkbox"
            checked={filters.show_noop || false}
            onChange={(e) => handleNoopToggle(e.target.checked)}
          />
          <span className="slider"></span>
        </label>
        <span className="switch-label">Show no-ops</span>
        <button className="bluebutton" type="submit">Save Filter</button>
        <button className="greybutton" type="button" onClick={clearFilters}>Clear</button>
      </form>
    </div>
  );
}

interface UnifiedEntryProps {
  entry: LogEntrySummary | Task;
  entries?: LogEntrySummary[];
  isTask: boolean;
  onClick?: (entry: LogEntrySummary) => void;
  isSelectMode?: boolean;
  isSelected?: boolean;
  onSelectionChange?: (uuid: string, isSelected: boolean) => void;
  selectedEntries?: string[];
  isExpanded?: boolean;
  onExpandChange?: (expanded: boolean) => void;
}

interface SingleIndexEntryProps {
  entry: LogEntrySummary;
  onClick: (entry: LogEntrySummary) => void;
  isSelectMode?: boolean;
  isSelected?: boolean;
  onSelectionChange?: (uuid: string, isSelected: boolean) => void;
  isTaskChild?: boolean;
}

function SingleIndexEntry({
  entry,
  onClick,
  isSelectMode = false,
  isSelected = false,
  onSelectionChange,
  isTaskChild = false
}: SingleIndexEntryProps) {
  const handleClick = (e: React.MouseEvent) => {
    if (isSelectMode) {
      e.stopPropagation();
      onSelectionChange?.(entry.uuid, !isSelected);
    } else {
      onClick(entry);
    }
  };

  return (
    <tr
      className={`
        ${isSelected ? 'selected' : ''}
        ${entry.is_noop ? 'noop-row' : ''}
      `.trim()}
      onClick={handleClick}
    >
      <td>
        {entry.capture_type === 'Run' ? <RunIcon /> : <EditIcon />}
      </td>
      {isSelectMode && (
        <td onClick={(e) => e.stopPropagation()}>
          <input
            type="checkbox"
            checked={isSelected}
            onChange={(e) => {
              e.stopPropagation();
              onSelectionChange?.(entry.uuid, e.target.checked);
            }}
          />
        </td>
      )}
      <td>
        <div className={`entry-status ${entry.exit_code === 0 ? 'success' : 'error'}`}>
          {entry.exit_code === 0 ? '✓' : '✗'}
        </div>
      </td>
      <td className="entry-time">{api.formatTimestamp(entry.start_time)}</td>
      <td>
        <div className="entry-content">
          <div className="entry-details">
            <div className="entry-host">{entry.host}</div>
            <div className="entry-cmd">{entry.cmd}</div>
            {entry.message && (
              <div className="entry-message">{entry.message}</div>
            )}
          </div>
        </div>
      </td>
      <td>
        <div className="button-group">
          <CopyButton entry={entry} />
        </div>
      </td>
      <td>{api.formatDuration(entry.duration_ms)}</td>
    </tr>
  );
}

function UnifiedEntry({ 
  entry, 
  entries = [], 
  isTask,
  onClick,
  isSelectMode = false,
  isSelected = false,
  onSelectionChange,
  selectedEntries = [],
  isExpanded: forcedExpanded,
  onExpandChange
}: UnifiedEntryProps) {
  const [localExpanded, setLocalExpanded] = useState(false);
  const isExpanded = forcedExpanded !== undefined ? forcedExpanded : localExpanded;

  const handleExpandClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onExpandChange) {
      onExpandChange(!isExpanded);
    } else {
      setLocalExpanded(!isExpanded);
    }
  };

  if (!isTask) {
    return (
      <tbody>
        <SingleIndexEntry
          entry={entry as LogEntrySummary}
          onClick={(entry) => onClick?.(entry)}
          isSelectMode={isSelectMode}
          isSelected={isSelected}
          onSelectionChange={onSelectionChange}
          isTaskChild={false}
        />
      </tbody>
    );
  }

  const task = entry as Task;
  const taskEntries = entries;
  
  // Calculate task summary info
  const hosts = [...new Set(taskEntries.map(e => e.host))];
  const startTime = new Date(Math.min(...taskEntries.map(e => new Date(e.start_time).getTime())));
  const endTime = new Date(Math.max(...taskEntries.map(e => new Date(e.start_time).getTime() + e.duration_ms)));

  const totalDuration = endTime.getTime() - startTime.getTime();
  
  // Calculate sum of individual entry durations for the summary
  const sumOfEntryDurations = taskEntries.reduce((sum, entry) => sum + entry.duration_ms, 0);
  
  // Check if any entry in the task failed
  const hasFailure = taskEntries.some(e => e.exit_code !== 0);

  return (
    <tbody className="unified-entry task-entry">
      <tr className="main-row" onClick={handleExpandClick}>
        <td>
          <div className="task-expand-icon">
            {isExpanded ? <ExpandedIcon /> : <CollapsedIcon />}
          </div>
        </td>
        {isSelectMode && (
          <td>
            {/* Remove checkbox for task entries */}
          </td>
        )}
        <td>
          <div className={`entry-status ${hasFailure ? 'error' : 'success'}`}>
            {hasFailure ? '✗' : '✓'}
          </div>
        </td>
        <td className="entry-time">{api.formatTimestamp(startTime.toISOString())}</td>
        <td>
          <div className="entry-content">
            <div className="entry-details">
              <div className="entry-host">{hosts.join(', ')}</div>
              <div className="entry-cmd">{task.name}</div>
              <div className="entry-summary">
                {taskEntries.length} entries • total duration {api.formatDuration(sumOfEntryDurations)}
              </div>
            </div>
          </div>
        </td>
        <td>
          <div className="button-group">
          </div>
        </td>
        <td>{api.formatDuration(totalDuration)}</td>
      </tr>
      {isExpanded && taskEntries.map(subEntry => (
        <SingleIndexEntry
          key={subEntry.uuid}
          entry={subEntry}
          onClick={(entry) => onClick?.(entry)}
          isSelectMode={isSelectMode}
          isSelected={selectedEntries.includes(subEntry.uuid)}
          onSelectionChange={onSelectionChange}
          isTaskChild={true}
        />
      ))}
    </tbody>
  );
}

export default function IndexPage() {
  const [entries, setEntries] = useState<LogEntrySummary[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchParams, setSearchParams] = useSearchParams();
  const [selectedEntries, setSelectedEntries] = useState<string[]>([]);
  const [isSelectMode, setIsSelectMode] = useState(false);
  const [showTaskManager, setShowTaskManager] = useState(false);
  const navigate = useNavigate();
  const [expandedTasks, setExpandedTasks] = useState<Set<number>>(new Set());
  const allTaskIds = tasks.map(task => task.id);
  const allExpanded = allTaskIds.length > 0 && allTaskIds.every(id => expandedTasks.has(id));
  
  // Parse filters from URL
  const filters: Filters = {
    date_from: searchParams.get('date_from') || undefined,
    date_to: searchParams.get('date_to') || undefined,
    host: searchParams.get('host') || undefined,
    search: searchParams.get('search') || undefined,
    search_content: searchParams.get('search_content') || undefined,
    show_noop: searchParams.get('show_noop') === 'true' || undefined,
  };

  // Update URL when filters change
  const updateFilters = (newFilters: Filters) => {
    const params = new URLSearchParams();
    if (newFilters.date_from) params.set('date_from', newFilters.date_from);
    if (newFilters.date_to) params.set('date_to', newFilters.date_to);
    if (newFilters.host) params.set('host', newFilters.host);
    if (newFilters.search) params.set('search', newFilters.search);
    if (newFilters.search_content) params.set('search_content', newFilters.search_content);
    if (newFilters.show_noop) params.set('show_noop', 'true');
    
    setSearchParams(params);
  };

  // Load entries and tasks when URL changes
  useEffect(() => {
    const loadData = async () => {
      try {
        setLoading(true);
        const [entriesData, tasksData] = await Promise.all([
          api.getEntriesSummary(filters),
          api.getTasks()
        ]);
        setEntries(entriesData);
        setTasks(tasksData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load data');
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [searchParams]);

  const handleRowClick = (uuid: string) => {
    navigate(`/entry/${uuid}`);
  };

  const handleEntryClick = (entry: LogEntrySummary) => {
    navigate(`/entry/${entry.uuid}`);
  };

  const handleSelectionChange = (uuid: string, isSelected: boolean) => {
    setSelectedEntries(prev => 
      isSelected ? [...prev, uuid] : prev.filter(id => id !== uuid)
    );
  };

  const handleSelectAll = () => {
    const allUngroupedIds = ungroupedEntries.map(e => e.uuid);
    setSelectedEntries(prev => [...new Set([...prev, ...allUngroupedIds])]);
  };

  const handleDeselectAll = () => {
    setSelectedEntries([]);
  };

  const toggleSelectMode = () => {
    setIsSelectMode(!isSelectMode);
    if (isSelectMode) {
      setSelectedEntries([]);
    }
  };

  const handleTaskCreated = async () => {
    // Reload data after task creation
    try {
      const [entriesData, tasksData] = await Promise.all([
        api.getEntriesSummary(filters),
        api.getTasks()
      ]);
      setEntries(entriesData);
      setTasks(tasksData);
      setSelectedEntries([]);
      setShowTaskManager(false);
    } catch (err) {
      console.error('Error reloading data after task creation:', err);
    }
  };

  const handleTaskUpdated = async () => {
    // Reload data after task update
    try {
      const [entriesData, tasksData] = await Promise.all([
        api.getEntriesSummary(filters),
        api.getTasks()
      ]);
      setEntries(entriesData);
      setTasks(tasksData);
      setSelectedEntries([]);
    } catch (err) {
      console.error('Error reloading data after task update:', err);
    }
  };

  // Group entries by task
  const taskGroups: { [taskId: number]: LogEntrySummary[] } = {};
  const ungroupedEntries: LogEntrySummary[] = [];

  entries.forEach(entry => {
    if (entry.task_id && entry.task_id > 0) {
      if (!taskGroups[entry.task_id]) {
        taskGroups[entry.task_id] = [];
      }
      taskGroups[entry.task_id].push(entry);
    } else {
      ungroupedEntries.push(entry);
    }
  });

  // Sort entries within each task group by start time
  Object.keys(taskGroups).forEach(taskId => {
    taskGroups[Number(taskId)].sort((a, b) => 
      new Date(a.start_time).getTime() - new Date(b.start_time).getTime()
    );
  });

  // Create a unified sorted list of entries and tasks
  const createUnifiedList = () => {
    const taskGroups: { [taskId: number]: LogEntrySummary[] } = {};
    const ungroupedEntries: LogEntrySummary[] = [];

    // First group entries by task
    entries.forEach(entry => {
      if (entry.task_id && entry.task_id > 0) {
        if (!taskGroups[entry.task_id]) {
          taskGroups[entry.task_id] = [];
        }
        taskGroups[entry.task_id].push(entry);
      } else {
        ungroupedEntries.push(entry);
      }
    });

    // Create unified list
    const unifiedList: Array<{
      type: 'task' | 'entry';
      item: Task | LogEntrySummary;
      entries?: LogEntrySummary[];
      timestamp: string;
    }> = [];

    // Add tasks
    tasks.forEach(task => {
      const taskEntries = taskGroups[task.id];
      if (taskEntries && taskEntries.length > 0) {
        const earliestEntry = taskGroups[task.id].reduce((earliest, entry) => {
          return new Date(entry.start_time) < new Date(earliest.start_time) ? entry : earliest;
        }, taskGroups[task.id][0]);

        unifiedList.push({
          type: 'task',
          item: task,
          entries: taskEntries,
          timestamp: earliestEntry.start_time
        });
      }
    });

    // Add ungrouped entries
    ungroupedEntries.forEach(entry => {
      unifiedList.push({
        type: 'entry',
        item: entry,
        timestamp: entry.start_time
      });
    });

    // Sort by timestamp, newest first
    return unifiedList.sort((a, b) => 
      new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );
  };

  const unifiedList = createUnifiedList();

  const handleExpandToggle = () => {
    if (allExpanded) {
      setExpandedTasks(new Set());
    } else {
      setExpandedTasks(new Set(allTaskIds));
    }
  };

  const handleTaskExpand = (taskId: number, expanded: boolean) => {
    setExpandedTasks(prev => {
      const next = new Set(prev);
      if (expanded) {
        next.add(taskId);
      } else {
        next.delete(taskId);
      }
      return next;
    });
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
        <div className="header-controls">
          <div>
            <button className={`bluebutton ${isSelectMode ? 'active' : ''}`} onClick={toggleSelectMode}>
              {isSelectMode ? 'Cancel' : 'Task grouping'}
            </button>
                       
            {isSelectMode && (
              <>
              <button className="greybutton" onClick={handleSelectAll}>
                Select All
              </button>
              <button className="greybutton" onClick={handleDeselectAll}>
                Deselect All
              </button>
              <button className="bluebutton" onClick={() => setShowTaskManager(true)} disabled={selectedEntries.length === 0}>
                Manage Tasks ({selectedEntries.length})
              </button>
              </>
            )}
          </div>
          <button className="bluebutton" type="button" onClick={() => navigate('/redact')}>
            Bulk Redact Passwords
          </button>
        </div>
      </div>
      
      <FilterForm filters={filters} onFiltersChange={updateFilters} onSearchResults={setEntries} onExpandToggle={handleExpandToggle} allExpanded={allExpanded} />
      

      {showTaskManager && (
        <TaskManager
          selectedEntries={selectedEntries}
          entries={entries}
          onTaskCreated={handleTaskCreated}
          onTaskUpdated={handleTaskUpdated}
          onClose={() => setShowTaskManager(false)}
        />
      )}

      <div className="entries-container">
        <table>
          <thead>
            <tr>
              <th style={{width: '24px'}}></th>
              {isSelectMode && <th style={{width: '24px'}}></th>}
              <th style={{width: '24px'}}></th>
              <th style={{width: '170px'}}>Time</th>
              <th style={{width: 'auto'}}>Details</th>
              <th style={{width: '24px'}}></th>
              <th style={{width: '80px'}}>Duration</th>
            </tr>
          </thead>
          {unifiedList.map(item => (
            <UnifiedEntry
              key={item.type === 'task' ? `task-${(item.item as Task).id}` : `entry-${(item.item as LogEntrySummary).uuid}`}
              entry={item.item}
              entries={item.entries}
              isTask={item.type === 'task'}
              onClick={handleEntryClick}
              isSelectMode={isSelectMode}
              isSelected={item.type === 'task' 
                ? item.entries?.every(e => selectedEntries.includes(e.uuid))
                : selectedEntries.includes((item.item as LogEntrySummary).uuid)
              }
              onSelectionChange={handleSelectionChange}
              selectedEntries={selectedEntries}
              isExpanded={item.type === 'task' ? expandedTasks.has((item.item as Task).id) : undefined}
              onExpandChange={item.type === 'task' ? (expanded) => handleTaskExpand((item.item as Task).id, expanded) : undefined}
            />
          ))}
        </table>
      </div>
    </div>
  );
} 