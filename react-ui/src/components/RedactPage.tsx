import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api';

interface MessageProps {
  message: string;
  type: 'success' | 'error';
}

function Message({ message, type }: MessageProps) {
  return (
    <div className={`message ${type}`}>
      {message}
    </div>
  );
}

export default function RedactPage() {
  const navigate = useNavigate();
  const [passwords, setPasswords] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState<{ text: string; type: 'success' | 'error' } | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const trimmedPasswords = passwords.trim();
    if (!trimmedPasswords) {
      setMessage({ text: 'Please enter at least one password to redact.', type: 'error' });
      return;
    }
    
    const passwordList = trimmedPasswords.split('\n').map(line => line.trim()).filter(line => line);
    if (passwordList.length === 0) {
      setMessage({ text: 'Please enter at least one valid password to redact.', type: 'error' });
      return;
    }
    
    const confirmMessage = `Are you sure you want to redact ${passwordList.length} password(s) from all log entries? This operation will permanently modify your log data and cannot be undone.`;
    if (!confirm(confirmMessage)) {
      return;
    }
    
    setIsLoading(true);
    
    try {
      const result = await api.bulkRedact({ passwords: passwordList });
      setMessage({ text: result.message || 'Passwords redacted successfully', type: 'success' });
      setPasswords('');
    } catch (error) {
      setMessage({ 
        text: 'Error performing redaction: ' + (error instanceof Error ? error.message : 'Unknown error'), 
        type: 'error' 
      });
    } finally {
      setIsLoading(false);
    }
  };

  const clearPasswords = () => {
    setPasswords('');
    setMessage(null);
  };

  return (
    <div className="container">
      <div className="header">
        <h1>Redact Passwords</h1>
        <button className="bluebutton" type="button" onClick={() => navigate('/')}>
          ← Back to list
        </button>
      </div>
      
      {message && (
        <Message message={message.text} type={message.type} />
      )}
      
      <div className="section">
        <p>Enter passwords to redact from all log entries. Each password should be on a separate line.</p>
        <p><strong>Warning:</strong> This operation will permanently modify your log data. Make sure you have a backup.</p>
        
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="passwords">Passwords to redact (one per line):</label>
            <textarea
              id="passwords"
              value={passwords}
              onChange={(e) => setPasswords(e.target.value)}
              rows={10}
              placeholder="password123&#10;secret456&#10;mytoken789"
              disabled={isLoading}
            />
          </div>
          <div className="form-group">
            <button 
              className="redbutton" 
              type="submit" 
              disabled={isLoading}
            >
              {isLoading ? 'Redacting...' : 'Redact Passwords'}
            </button>
            <button 
              className="greybutton" 
              type="button" 
              onClick={clearPasswords}
              disabled={isLoading}
            >
              Clear
            </button>
          </div>
        </form>
      </div>
    </div>
  );
} 