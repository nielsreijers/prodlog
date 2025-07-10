import React, { useState, useEffect } from 'react';
import { Task, TaskCreateRequest, TaskUpdateRequest, LogEntrySummary } from '../types';
import { api } from '../api';

interface TaskManagerProps {
  selectedEntries: string[];
  entries: LogEntrySummary[];
  onTaskCreated?: () => void;
  onTaskUpdated?: () => void;
  onClose?: () => void;
}

export const TaskManager: React.FC<TaskManagerProps> = ({
  selectedEntries,
  entries,
  onTaskCreated,
  onTaskUpdated,
  onClose,
}) => {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [newTaskName, setNewTaskName] = useState('');
  const [selectedTask, setSelectedTask] = useState<number | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTasks();
  }, []);

  const loadTasks = async () => {
    try {
      const tasks = await api.getTasks();
      setTasks(tasks);
    } catch (err) {
      setError('Failed to load tasks');
    }
  };

  const handleCreateTask = async () => {
    if (!newTaskName.trim()) {
      setError('Task name cannot be empty');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const request: TaskCreateRequest = {
        name: newTaskName.trim(),
        entry_uuids: selectedEntries
      };

      await api.createTask(request);
      setNewTaskName('');
      await loadTasks();
      onTaskCreated?.();
    } catch (err) {
      setError('Failed to create task');
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddToExistingTask = async () => {
    if (!selectedTask) {
      setError('Please select a task');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const request: TaskUpdateRequest = {
        task_id: selectedTask,
        entry_uuids: selectedEntries
      };

      await api.updateTask(request);
      await loadTasks();
      onTaskUpdated?.();
    } catch (err) {
      setError('Failed to add entries to task');
    } finally {
      setIsLoading(false);
    }
  };

  const handleUngroupEntries = async () => {
    setIsLoading(true);
    setError(null);

    try {
      await api.ungroupEntries(selectedEntries);
      await loadTasks();
      onTaskUpdated?.();
    } catch (err) {
      setError('Failed to ungroup entries');
    } finally {
      setIsLoading(false);
    }
  };

  const getSelectedEntriesInfo = () => {
    const selectedEntryObjects = entries.filter(entry => selectedEntries.includes(entry.uuid));
    const hosts = [...new Set(selectedEntryObjects.map(e => e.host))];
    const commands = selectedEntryObjects.map(e => e.cmd.length > 50 ? e.cmd.substring(0, 50) + '...' : e.cmd);
    
    return {
      count: selectedEntries.length,
      hosts: hosts.join(', '),
      commands: commands.slice(0, 3),
      hasMore: commands.length > 3
    };
  };

  if (selectedEntries.length === 0) {
    return (
      <div className="task-manager">
        <div className="task-manager-header">
          <h3>Task Manager</h3>
          {onClose && (
            <button onClick={onClose} className="close-button">×</button>
          )}
        </div>
        <div className="task-manager-content">
          <p>Select entries to group them into tasks</p>
        </div>
      </div>
    );
  }

  const info = getSelectedEntriesInfo();

  return (
    <div className="task-manager">
      <div className="task-manager-header">
        <h3>Task Manager</h3>
        {onClose && (
          <button onClick={onClose} className="close-button">×</button>
        )}
      </div>
      
      <div className="task-manager-content">
        <div className="selected-entries-info">
          <h4>Selected Entries ({info.count})</h4>
          <p><strong>Hosts:</strong> {info.hosts}</p>
          <div>
            <strong>Commands:</strong>
            <ul>
              {info.commands.map((cmd, idx) => (
                <li key={idx}>{cmd}</li>
              ))}
              {info.hasMore && <li>... and {info.count - 3} more</li>}
            </ul>
          </div>
        </div>

        {error && (
          <div className="error-message">
            {error}
          </div>
        )}

        <div className="task-actions">
          <div className="create-new-task">
            <h4>Create New Task</h4>
            <div className="task-input-group">
              <input
                type="text"
                placeholder="Task name"
                value={newTaskName}
                onChange={(e) => setNewTaskName(e.target.value)}
                disabled={isLoading}
              />
              <button 
                onClick={handleCreateTask} 
                disabled={isLoading || !newTaskName.trim()}
              >
                {isLoading ? 'Creating...' : 'Create Task'}
              </button>
            </div>
          </div>

          {tasks.length > 0 && (
            <div className="add-to-existing-task">
              <h4>Add to Existing Task</h4>
              <div className="task-input-group">
                <select
                  value={selectedTask || ''}
                  onChange={(e) => setSelectedTask(Number(e.target.value) || null)}
                  disabled={isLoading}
                >
                  <option value="">Select a task...</option>
                  {tasks.map(task => (
                    <option key={task.id} value={task.id}>
                      {task.name}
                    </option>
                  ))}
                </select>
                <button 
                  onClick={handleAddToExistingTask} 
                  disabled={isLoading || !selectedTask}
                >
                  {isLoading ? 'Adding...' : 'Add to Task'}
                </button>
              </div>
            </div>
          )}

          <div className="ungroup-entries">
            <h4>Ungroup Entries</h4>
            <button 
              onClick={handleUngroupEntries} 
              disabled={isLoading}
              className="ungroup-button"
            >
              {isLoading ? 'Ungrouping...' : 'Remove from Tasks'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}; 