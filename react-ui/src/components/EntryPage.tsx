import React, { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import { LogEntry } from '../types';
import { api } from '../api';

interface MessageProps {
  message: string;
  type: 'success' | 'error';
}

function Message({ message, type }: MessageProps) {
  return (
    <div className={`message ${type} temp-message`}>
      {message}
    </div>
  );
}

interface EditSectionsProps {
  entry: LogEntry;
  onEntryUpdate: (entry: LogEntry) => void;
  onMessage: (message: string, type: 'success' | 'error') => void;
}

function EditSections({ entry, onEntryUpdate, onMessage }: EditSectionsProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [message, setMessage] = useState(entry.message);
  const [isNoop, setIsNoop] = useState(entry.is_noop);
  const [password, setPassword] = useState('');

  const handleCommentSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    try {
      setIsUpdating(true);
      await api.updateEntry({
        uuid: entry.uuid,
        message,
        is_noop: entry.is_noop
      });
      
      const updatedEntry = { ...entry, message };
      onEntryUpdate(updatedEntry);
      onMessage('Comment updated successfully', 'success');
    } catch (error) {
      onMessage('Error updating comment: ' + (error instanceof Error ? error.message : 'Unknown error'), 'error');
    } finally {
      setIsUpdating(false);
    }
  };

  const handleNoopToggle = async (checked: boolean) => {
    const newNoopStatus = checked;
    const originalNoopStatus = entry.is_noop;
    
    setIsNoop(newNoopStatus);
    setIsUpdating(true);
    
    try {
      await api.updateEntry({
        uuid: entry.uuid,
        message: entry.message,
        is_noop: newNoopStatus
      });
      
      const updatedEntry = { ...entry, is_noop: newNoopStatus };
      onEntryUpdate(updatedEntry);
      
      const action = newNoopStatus ? 'NO-OP' : 'NOT a no-op';
      onMessage(`Entry successfully marked as ${action}.`, 'success');
    } catch (error) {
      // Revert to original state on error
      setIsNoop(originalNoopStatus);
      const action = newNoopStatus ? 'NO-OP' : 'NOT a no-op';
      onMessage(`Failed to mark entry as ${action}.`, 'error');
    } finally {
      setIsUpdating(false);
    }
  };

  const handleRedactSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!password.trim()) {
      onMessage('Please enter a password to redact.', 'error');
      return;
    }

    if (!confirm(`Are you sure you want to redact the password "${password}" from this entry? This operation will permanently modify the entry data and cannot be undone.`)) {
      return;
    }

    try {
      const result = await api.redactEntry({
        uuid: entry.uuid,
        password: password.trim()
      });
      
      onMessage(result.message || 'Password redacted successfully', 'success');
      setPassword('');
    } catch (error) {
      onMessage('Error redacting password: ' + (error instanceof Error ? error.message : 'Unknown error'), 'error');
    }
  };

  const toggleVisibility = () => {
    setIsVisible(!isVisible);
  };

  return (
    <>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
        <button className="bluebutton" type="button" onClick={() => window.history.back()}>
          ← Back to list
        </button>
        <button className="bluebutton" onClick={toggleVisibility}>
          <span>{isVisible ? '▲' : '▼'}</span>
          <span>Edit</span>
        </button>
      </div>

      {isVisible && (
        <div className="edit-sections-container">
          <div className="section comment-section">
            <h2>Comment</h2>
            <form onSubmit={handleCommentSubmit}>
              <div>
                <textarea
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  rows={5}
                  disabled={isUpdating}
                />
              </div>
              <div className="button-group">
                <button type="submit" className="bluebutton" disabled={isUpdating}>
                  {isUpdating ? 'Updating...' : 'Update Comment'}
                </button>
              </div>
            </form>
          </div>

          <div className="section noop-section">
            <h2>No-op</h2>
            <div className="noop-toggle-container">
              <div className="switch-container">
                <label className="switch">
                  <input
                    type="checkbox"
                    checked={isNoop}
                    onChange={(e) => handleNoopToggle(e.target.checked)}
                    disabled={isUpdating}
                  />
                  <span className="slider"></span>
                </label>
                <span className="switch-label">
                  {isUpdating ? 'Updating...' : 
                   isNoop ? 'Marked as no-op. This entry had no effect.' : 'Not a no-op.'}
                </span>
              </div>
            </div>
          </div>

          <div className="section redact-section">
            <h2>Redact Password</h2>
            <p>Remove a password from this specific entry. This will replace all occurrences of the password with [REDACTED].</p>
            <form onSubmit={handleRedactSubmit}>
              <div>
                <label htmlFor="password">Password to redact:</label>
                <input
                  type="text"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter password to redact"
                />
              </div>
              <div className="button-group">
                <button type="submit" className="redbutton">
                  Redact Password
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </>
  );
}

interface OutputDisplayProps {
  entry: LogEntry;
}

function OutputDisplay({ entry }: OutputDisplayProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminalInstance = useRef<Terminal | null>(null);

  useEffect(() => {
    if (!terminalRef.current) return;

    // Initialize terminal if not already done
    if (!terminalInstance.current) {
      terminalInstance.current = new Terminal({
        cols: entry.terminal_cols || 120,
        rows: entry.terminal_rows || 40,
        cursorBlink: true,
        scrollback: 9999999,
        fontSize: 14,
        fontFamily: 'monospace',
        convertEol: true,
        disableStdin: true
      });

      // Don't let xterm.js handle any key events
      terminalInstance.current.attachCustomKeyEventHandler(() => false);
      terminalInstance.current.open(terminalRef.current);
    }

    try {
      // Decode base64 to raw bytes
      const binaryString = atob(entry.captured_output);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      
      // Write raw bytes to terminal
      terminalInstance.current.write(bytes);
    } catch (error) {
      console.error('Error decoding output:', error);
      if (terminalInstance.current) {
        terminalInstance.current.write('Error decoding output');
      }
    }

    // Cleanup function
    return () => {
      if (terminalInstance.current) {
        terminalInstance.current.dispose();
        terminalInstance.current = null;
      }
    };
  }, [entry.captured_output, entry.terminal_cols, entry.terminal_rows]);

  return (
    <div className="section">
      <h2>Output</h2>
      <div 
        ref={terminalRef}
        style={{
          backgroundColor: '#000',
          padding: '1rem',
          borderRadius: '8px',
          minHeight: '200px'
        }}
      />
    </div>
  );
}

interface DiffDisplayProps {
  entry: LogEntry;
}

function DiffDisplay({ entry }: DiffDisplayProps) {
  const [diffContent, setDiffContent] = useState<string>('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadDiff = async () => {
      try {
        const result = await api.getDiffContent(entry.uuid);
        setDiffContent(result.diff);
      } catch (error) {
        setDiffContent('Error loading diff content');
      } finally {
        setLoading(false);
      }
    };

    loadDiff();
  }, [entry.uuid]);

  return (
    <div className="section">
      <h2>File Diff</h2>
      {loading ? (
        <div>Loading diff...</div>
      ) : (
        <pre 
          className="diff-output"
          dangerouslySetInnerHTML={{ __html: diffContent }}
        />
      )}
    </div>
  );
}

export default function EntryPage() {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const [entry, setEntry] = useState<LogEntry | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<{ text: string; type: 'success' | 'error' } | null>(null);

  useEffect(() => {
    if (!uuid) {
      navigate('/');
      return;
    }

    const loadEntry = async () => {
      try {
        const data = await api.getEntry(uuid);
        setEntry(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load entry');
      } finally {
        setLoading(false);
      }
    };

    loadEntry();
  }, [uuid, navigate]);

  const handleMessage = (text: string, type: 'success' | 'error') => {
    setMessage({ text, type });
  };

  const handleEntryUpdate = (updatedEntry: LogEntry) => {
    setEntry(updatedEntry);
  };

  if (loading) {
    return (
      <div className="container">
        <div>Loading...</div>
      </div>
    );
  }

  if (error || !entry) {
    return (
      <div className="container">
        <div className="message error">
          Error: {error || 'Entry not found'}
        </div>
      </div>
    );
  }

  return (
    <div className="container">
      {message && (
        <Message message={message.text} type={message.type} />
      )}

      <EditSections 
        entry={entry} 
        onEntryUpdate={handleEntryUpdate}
        onMessage={handleMessage}
      />

      <div className="section">
        <h2>{entry.capture_type === 'Run' ? entry.cmd : entry.filename}</h2>
        <div className="info-grid">
          <div className="info-item">
            <span className="info-label">Host:</span>
            <span className="info-value">{entry.host}</span>
          </div>
          <div className="info-item">
            <span className="info-label">Command:</span>
            <span className="info-value">{entry.cmd}</span>
          </div>
          <div className="info-item">
            <span className="info-label">Directory:</span>
            <span className="info-value">{entry.cwd}</span>
          </div>
          <div className="info-item">
            <span className="info-label">Start:</span>
            <span className="info-value">{api.formatTimestamp(entry.start_time)}</span>
          </div>
          <div className="info-item">
            <span className="info-label">End:</span>
            <span className="info-value">{api.formatTimestamp(new Date(new Date(entry.start_time).getTime() + entry.duration_ms).toISOString())}</span>
          </div>
          <div className="info-item">
            <span className="info-label">Duration:</span>
            <span className="info-value">{api.formatDuration(entry.duration_ms)}</span>
          </div>
          <div className="info-item">
            <span className="info-label">Exit Code:</span>
            <span className="info-value">{entry.exit_code}</span>
          </div>
        </div>
        {entry.message && (
          <div className="message">
            {entry.message}
          </div>
        )}
      </div>

      {entry.capture_type === 'Run' ? (
        <OutputDisplay entry={entry} />
      ) : (
        <DiffDisplay entry={entry} />
      )}
    </div>
  );
} 