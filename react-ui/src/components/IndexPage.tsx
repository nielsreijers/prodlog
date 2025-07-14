import React, { useState, useEffect, useRef, useMemo } from 'react';
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

interface ActiveTaskButtonProps {
  taskId: number;
  isActive: boolean;
  onToggle: (taskId: number) => void;
}

function ActiveTaskButton({ taskId, isActive, onToggle }: ActiveTaskButtonProps) {
  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onToggle(taskId);
  };

  return (
    <button 
      className={`active-task-button ${isActive ? 'active' : ''}`}
      onClick={handleClick}
      title={isActive ? 'Deactivate task' : 'Activate task'}
    >
      {isActive ? '🔴' : '🔘'}
    </button>
  );
}

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
  const [currentPreset, setCurrentPreset] = useState<string | null>(filters.date_range || null);
  
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
    setCurrentPreset(filters.date_range || null);
  }, [filters.host, filters.search, filters.search_content, filters.date_from, filters.date_to, filters.date_range]);
  
  // Debounced effect to search without updating URL
  useEffect(() => {
    // Check if local values differ from current URL filters
    const currentFilters = filtersRef.current;
    
    // Only search if local values are different from URL
    const hasChanged = 
      localDateRange.from !== currentFilters.date_from ||
      localDateRange.to !== currentFilters.date_to ||
      currentPreset !== currentFilters.date_range ||
      localHost !== currentFilters.host ||
      localSearch !== currentFilters.search ||
      localSearchContent !== currentFilters.search_content;
    
    if (!hasChanged) return;
    
    const timeoutId = setTimeout(() => {
      // Call API directly without updating URL
      const searchFilters = {
        date_from: localDateRange.from || undefined,
        date_to: localDateRange.to || undefined,
        date_range: currentPreset || undefined,
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
  }, [localHost, localSearch, localSearchContent, localDateRange.from, localDateRange.to, currentPreset]);
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Force immediate update on form submit
    onFiltersChange({
      date_from: currentPreset ? undefined : (localDateRange.from || undefined),
      date_to: currentPreset ? undefined : (localDateRange.to || undefined),
      date_range: currentPreset || undefined,
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
    setCurrentPreset(null);
    onFiltersChange({});
  };

  const handleDateRangeChange = (range: { from: string; to: string }, preset?: string) => {
    setLocalDateRange(range);
    setCurrentPreset(preset || null);
  };

  return (
    <div className="filters">
      <form onSubmit={handleSubmit}>
        <button className="greybutton  filters-right" onClick={onExpandToggle}>
          {allExpanded ? 'Collapse All' : 'Expand All'}
        </button>
        <DateRangePicker
          value={localDateRange}
          onChange={handleDateRangeChange}
          currentPreset={currentPreset}
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
  activeTaskId: number | null;
  onActiveTaskToggle: (taskId: number) => void;
  firstTime?: string;
  lastTime?: string;
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
  isTaskChild = false,
  showTaskColumn = false,
  task = null
}: SingleIndexEntryProps & { showTaskColumn?: boolean; task?: Task | null }) {
  const handleClick = (e: React.MouseEvent) => {
    if (isSelectMode) {
      e.stopPropagation();
      onSelectionChange?.(entry.uuid, !isSelected);
    } else {
      // Check if Ctrl/Cmd key is pressed for new tab
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        window.open(`/entry/${entry.uuid}`, '_blank');
      } else {
        onClick(entry);
      }
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
      {showTaskColumn && (
        <td className="entry-task">
          {task ? (
            <div className="task-name-small">{task.name}</div>
          ) : (
            <div className="no-task">—</div>
          )}
        </td>
      )}
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
  onExpandChange,
  activeTaskId,
  onActiveTaskToggle,
  firstTime,
  lastTime
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
    <tbody className={`unified-entry task-entry ${activeTaskId === task.id ? 'active-task' : ''}`}>
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
        <td className="entry-time">
          {isTask && firstTime && lastTime ? (
            <div className="task-time-range">
              <div>{api.formatTimestamp(firstTime)}</div>
              <div>to {api.formatTimestamp(lastTime)}</div>
            </div>
          ) : (
            api.formatTimestamp(startTime.toISOString())
          )}
        </td>
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
            <ActiveTaskButton 
              taskId={task.id} 
              isActive={activeTaskId === task.id}
              onToggle={onActiveTaskToggle}
            />
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

function ActiveTaskMessage({ activeTaskId, tasks }: { activeTaskId: number | null; tasks: Task[] }) {
  if (!activeTaskId) return null;
  
  const activeTask = tasks.find(task => task.id === activeTaskId);
  if (!activeTask) return null;

  return (
    <div className="active-task-header-message">
      🔴 <strong>Active Task:</strong> {activeTask.name}
    </div>
  );
}

interface StartTaskButtonProps {
  onCreateAndActivate: (taskName: string) => void;
}

function StartTaskButton({ onCreateAndActivate }: StartTaskButtonProps) {
  const handleClick = async () => {
    try {
      const taskName = prompt("Enter task name:");
      if (!taskName || taskName.trim() === '') {
        return; // User cancelled or entered empty name
      }
      
      onCreateAndActivate(taskName.trim());
    } catch (error) {
      console.error('Error starting task:', error);
    }
  };

  return (
    <button className="bluebutton" onClick={handleClick}>Start New Task</button>
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
  const [activeTaskId, setActiveTaskId] = useState<number | null>(null);
  const navigate = useNavigate();
  const isInitialLoad = useRef(true);
  
  // Parse expanded tasks from URL
  const expandedTasksParam = searchParams.get('expanded');
  const expandedTasksFromUrl = expandedTasksParam 
    ? new Set(expandedTasksParam.split(',').map(id => parseInt(id)).filter(id => !isNaN(id)))
    : new Set<number>();
  
  const [expandedTasks, setExpandedTasks] = useState<Set<number>>(expandedTasksFromUrl);
  const allTaskIds = tasks.map(task => task.id);
  const allExpanded = allTaskIds.length > 0 && allTaskIds.every(id => expandedTasks.has(id));
  
  // Parse filters from URL
  const filters = useMemo(() => ({
    date_from: searchParams.get('date_from') || undefined,
    date_to: searchParams.get('date_to') || undefined,
    date_range: searchParams.get('date_range') || undefined,
    host: searchParams.get('host') || undefined,
    search: searchParams.get('search') || undefined,
    search_content: searchParams.get('search_content') || undefined,
    show_noop: searchParams.get('show_noop') === 'true' || undefined,
  }), [searchParams]);

  // Parse view mode from URL
  const isFlatView = searchParams.get('view') === 'flat';

  // Update URL when filters change
  const updateFilters = (newFilters: Filters) => {
    const params = new URLSearchParams();
    
    // Prioritize date_range over explicit dates
    if (newFilters.date_range) {
      params.set('date_range', newFilters.date_range);
    } else {
      if (newFilters.date_from) params.set('date_from', newFilters.date_from);
      if (newFilters.date_to) params.set('date_to', newFilters.date_to);
    }
    
    if (newFilters.host) params.set('host', newFilters.host);
    if (newFilters.search) params.set('search', newFilters.search);
    if (newFilters.search_content) params.set('search_content', newFilters.search_content);
    if (newFilters.show_noop) params.set('show_noop', 'true');
    
    setSearchParams(params);
  };

  // Update URL when view mode changes
  const updateViewMode = (flatView: boolean) => {
    const params = new URLSearchParams(searchParams);
    if (flatView) {
      params.set('view', 'flat');
    } else {
      params.delete('view');
    }
    setSearchParams(params);
  };

  // Update URL when expanded tasks change
  const updateExpandedTasks = (newExpandedTasks: Set<number>) => {
    const params = new URLSearchParams(searchParams);
    if (newExpandedTasks.size > 0) {
      params.set('expanded', Array.from(newExpandedTasks).join(','));
    } else {
      params.delete('expanded');
    }
    setSearchParams(params);
  };

  // Load entries and tasks when URL changes
  useEffect(() => {
    const loadData = async () => {
      try {
        setLoading(true);
        const [entriesData, tasksData, activeTaskData] = await Promise.all([
          api.getEntriesSummary(filters),
          api.getTasks(),
          api.getActiveTask()
        ]);
        setEntries(entriesData);
        setTasks(tasksData);
        setActiveTaskId(activeTaskData.task_id);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load data');
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [searchParams]);

  // Update URL when expanded tasks change
  useEffect(() => {
    if (!isInitialLoad.current) {
      updateExpandedTasks(expandedTasks);
    }
    isInitialLoad.current = false;
  }, [expandedTasks]);

  const handleActiveTaskToggle = async (taskId: number) => {
    try {
      const newActiveTaskId = activeTaskId === taskId ? null : taskId;
      await api.setActiveTask(newActiveTaskId);
      setActiveTaskId(newActiveTaskId);
    } catch (err) {
      console.error('Error toggling active task:', err);
    }
  };

  const handleStartTask = async (taskName: string) => {
    try {
      // Create the task
      const result = await api.createTask({
        name: taskName,
        entry_uuids: []
      });
      
      // Set it as the active task
      await api.setActiveTask(result.task_id);
      
      // Reload data to get the new task
      const [entriesData, tasksData, activeTaskData] = await Promise.all([
        api.getEntriesSummary(filters),
        api.getTasks(),
        api.getActiveTask()
      ]);
      setEntries(entriesData);
      setTasks(tasksData);
      setActiveTaskId(activeTaskData.task_id);
    } catch (err) {
      console.error('Error starting task:', err);
    }
  };

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
      firstTime?: string;
      lastTime?: string;
    }> = [];

    // Add tasks
    tasks.forEach(task => {
      const taskEntries = taskGroups[task.id];
      if (taskEntries && taskEntries.length > 0) {
        const earliestEntry = taskGroups[task.id].reduce((earliest, entry) => {
          return new Date(entry.start_time) < new Date(earliest.start_time) ? entry : earliest;
        }, taskGroups[task.id][0]);
        
        const latestEntry = taskGroups[task.id].reduce((latest, entry) => {
          return new Date(entry.start_time) > new Date(latest.start_time) ? entry : latest;
        }, taskGroups[task.id][0]);

        unifiedList.push({
          type: 'task',
          item: task,
          entries: taskEntries,
          timestamp: latestEntry.start_time, // Use latest time for sorting
          firstTime: earliestEntry.start_time,
          lastTime: latestEntry.start_time
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

  // Create a flat list of all entries with task information
  const createFlatList = () => {
    const flatEntries = entries.map(entry => {
      const task = entry.task_id ? tasks.find(t => t.id === entry.task_id) : null;
      return {
        entry,
        task,
        timestamp: entry.start_time
      };
    });

    // Sort by timestamp, newest first
    return flatEntries.sort((a, b) => 
      new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );
  };

  const flatList = createFlatList();

  const handleExpandToggle = () => {
    if (allExpanded) {
      const newExpandedTasks = new Set<number>();
      setExpandedTasks(newExpandedTasks);
      updateExpandedTasks(newExpandedTasks);
    } else {
      const newExpandedTasks = new Set(allTaskIds);
      setExpandedTasks(newExpandedTasks);
      updateExpandedTasks(newExpandedTasks);
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
            <button className={`bluebutton ${isFlatView ? 'active' : ''}`} onClick={() => updateViewMode(!isFlatView)}>
              {isFlatView ? 'Grouped View' : 'Flat View'}
            </button>
            <StartTaskButton onCreateAndActivate={handleStartTask} />
            <button className={`bluebutton ${isSelectMode ? 'active' : ''}`} onClick={toggleSelectMode}>
              {isSelectMode ? 'Cancel' : 'Task Grouping'}
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
          <ActiveTaskMessage activeTaskId={activeTaskId} tasks={tasks} />
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
              <th style={{width: '26px'}}></th>
              {isSelectMode && <th style={{width: '24px'}}></th>}
              <th style={{width: '24px'}}></th>
              <th style={{width: '170px'}}>Time</th>
              <th style={{width: 'auto'}}>Details</th>
              {isFlatView && <th style={{width: '150px'}}>Task</th>}
              <th style={{width: '24px'}}></th>
              <th style={{width: '80px'}}>Duration</th>
            </tr>
          </thead>
          {isFlatView ? (
            flatList.map(item => (
              <tbody key={`entry-${item.entry.uuid}`}>
                <SingleIndexEntry
                  entry={item.entry}
                  onClick={handleEntryClick}
                  isSelectMode={isSelectMode}
                  isSelected={selectedEntries.includes(item.entry.uuid)}
                  onSelectionChange={handleSelectionChange}
                  showTaskColumn={true}
                  task={item.task}
                />
              </tbody>
            ))
          ) : (
            unifiedList.map(item => (
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
                activeTaskId={activeTaskId}
                onActiveTaskToggle={handleActiveTaskToggle}
                firstTime={item.firstTime}
                lastTime={item.lastTime}
              />
            ))
          )}
        </table>
      </div>
    </div>
  );
} 