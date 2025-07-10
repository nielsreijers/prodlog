import React, { useState } from 'react';
import { Task, LogEntrySummary } from '../types';
import { api } from '../api';

interface TaskGroupProps {
  task: Task;
  entries: LogEntrySummary[];
  onEntryClick?: (entry: LogEntrySummary) => void;
  onSelectionChange?: (selectedIds: string[]) => void;
  selectedEntries?: string[];
}

export const TaskGroup: React.FC<TaskGroupProps> = ({
  task,
  entries,
  onEntryClick,
  onSelectionChange,
  selectedEntries = [],
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isSelectMode, setIsSelectMode] = useState(false);

  const toggleExpanded = () => {
    setIsExpanded(!isExpanded);
  };

  const toggleSelectMode = () => {
    setIsSelectMode(!isSelectMode);
    if (isSelectMode) {
      // Clear selections when exiting select mode
      onSelectionChange?.([]);
    }
  };

  const handleEntrySelection = (uuid: string, isSelected: boolean) => {
    if (!onSelectionChange) return;
    
    const newSelection = isSelected
      ? [...selectedEntries, uuid]
      : selectedEntries.filter(id => id !== uuid);
    
    onSelectionChange(newSelection);
  };

  const selectAllInTask = () => {
    if (!onSelectionChange) return;
    
    const allTaskEntryIds = entries.map(e => e.uuid);
    const newSelection = [...new Set([...selectedEntries, ...allTaskEntryIds])];
    onSelectionChange(newSelection);
  };

  const deselectAllInTask = () => {
    if (!onSelectionChange) return;
    
    const taskEntryIds = entries.map(e => e.uuid);
    const newSelection = selectedEntries.filter(id => !taskEntryIds.includes(id));
    onSelectionChange(newSelection);
  };

  const getTaskSummary = () => {
    const hosts = [...new Set(entries.map(e => e.host))];
    const startTime = entries.reduce((earliest, entry) => {
      const entryTime = new Date(entry.start_time);
      return entryTime < earliest ? entryTime : earliest;
    }, new Date(entries[0].start_time));
    
    const endTime = entries.reduce((latest, entry) => {
      const entryEndTime = new Date(new Date(entry.start_time).getTime() + entry.duration_ms);
      return entryEndTime > latest ? entryEndTime : latest;
    }, new Date(entries[0].start_time));

    const totalDuration = endTime.getTime() - startTime.getTime();
    
    return {
      hosts: hosts.join(', '),
      startTime: api.formatTimestamp(startTime.toISOString()),
      endTime: api.formatTimestamp(endTime.toISOString()),
      duration: Math.round(totalDuration / 1000), // seconds
      entryCount: entries.length
    };
  };

  const summary = getTaskSummary();
  const allTaskEntriesSelected = entries.every(e => selectedEntries.includes(e.uuid));

  return (
    <div className="task-group">
      <div className="task-header" onClick={toggleExpanded}>
        <div className="task-expand-icon">
          {isExpanded ? '▼' : '▶'}
        </div>
        <div className="task-info">
          <div className="task-name">{task.name}</div>
          <div className="task-summary">
            {summary.entryCount} entries • {summary.hosts} • {summary.duration}s
          </div>
          <div className="task-time-range">
            {summary.startTime} → {summary.endTime}
          </div>
        </div>
        <div className="task-actions">
          <button 
            className="task-select-button"
            onClick={(e) => {
              e.stopPropagation();
              toggleSelectMode();
            }}
          >
            {isSelectMode ? 'Done' : 'Select'}
          </button>
        </div>
      </div>

      {isExpanded && (
        <div className="task-entries">
          {isSelectMode && (
            <div className="task-selection-controls">
              <button 
                onClick={allTaskEntriesSelected ? deselectAllInTask : selectAllInTask}
                className="select-all-button"
              >
                {allTaskEntriesSelected ? 'Deselect All' : 'Select All'} in Task
              </button>
            </div>
          )}
          
          {entries.map(entry => (
            <div 
              key={entry.uuid} 
              className={`task-entry ${selectedEntries.includes(entry.uuid) ? 'selected' : ''}`}
            >
              {isSelectMode && (
                <input
                  type="checkbox"
                  checked={selectedEntries.includes(entry.uuid)}
                  onChange={(e) => handleEntrySelection(entry.uuid, e.target.checked)}
                  className="entry-checkbox"
                />
              )}
              
              <div 
                className="entry-content"
                onClick={() => !isSelectMode && onEntryClick?.(entry)}
              >
                <div className="entry-time">
                  {api.formatTimestamp(entry.start_time)}
                </div>
                <div className="entry-details">
                  <div className="entry-host">{entry.host}</div>
                  <div className="entry-cmd">{entry.cmd}</div>
                  {entry.message && <div className="entry-message">{entry.message}</div>}
                </div>
                <div className="entry-meta">
                  <div className={`entry-status ${entry.exit_code === 0 ? 'success' : 'error'}`}>
                    {entry.exit_code === 0 ? '✓' : '✗'}
                  </div>
                  {entry.is_noop && <div className="entry-noop">No-op</div>}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}; 