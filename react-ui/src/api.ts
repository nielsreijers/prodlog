import { 
  LogEntry, 
  LogEntrySummary,
  Filters, 
  BulkRedactRequest, 
  EntryRedactRequest, 
  EntryUpdateRequest,
  DiffResponse,
  ApiResponse,
  Task,
  TaskCreateRequest,
  TaskUpdateRequest,
  TaskCreateResponse
} from './types';

// Convert preset date ranges to actual dates
function calculateDateRange(preset: string): { from: string; to: string } {
  const today = new Date();
  const formatDate = (date: Date) => date.toISOString().split('T')[0];
  
  switch (preset) {
    case 'today':
      return {
        from: formatDate(today),
        to: formatDate(today)
      };
    
    case 'yesterday': {
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);
      return {
        from: formatDate(yesterday),
        to: formatDate(yesterday)
      };
    }
    
    case 'last_7_days': {
      const last7Days = new Date(today);
      last7Days.setDate(last7Days.getDate() - 7);
      return {
        from: formatDate(last7Days),
        to: formatDate(today)
      };
    }
    
    case 'last_30_days': {
      const last30Days = new Date(today);
      last30Days.setDate(last30Days.getDate() - 30);
      return {
        from: formatDate(last30Days),
        to: formatDate(today)
      };
    }
    
    case 'last_365_days': {
      const last365Days = new Date(today);
      last365Days.setDate(last365Days.getDate() - 365);
      return {
        from: formatDate(last365Days),
        to: formatDate(today)
      };
    }
    
    case 'all_time':
      return {
        from: '',
        to: ''
      };
    
    default:
      return {
        from: '',
        to: ''
      };
  }
}

class ApiService {
  private baseUrl = '/api';

  // Helper method to resolve date ranges from filters
  private resolveDateRange(filters: Filters): { date_from?: string; date_to?: string } {
    // If we have a preset range, calculate the actual dates
    if (filters.date_range) {
      const { from, to } = calculateDateRange(filters.date_range);
      return {
        date_from: from || undefined,
        date_to: to || undefined
      };
    }
    
    // Otherwise use the explicit dates if provided
    return {
      date_from: filters.date_from,
      date_to: filters.date_to
    };
  }

  async get<T>(url: string): Promise<T> {
    const response = await fetch(this.baseUrl + url);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async post<T>(url: string, data: any): Promise<T> {
    const response = await fetch(this.baseUrl + url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    return response.json();
  }

  // Get entries with filters (full data)
  async getEntries(filters: Filters = {}): Promise<LogEntry[]> {
    const params = new URLSearchParams();
    
    // Resolve date range from preset or explicit dates
    const { date_from, date_to } = this.resolveDateRange(filters);
    
    if (date_from) params.append('date_from', date_from);
    if (date_to) params.append('date_to', date_to);
    if (filters.host) params.append('host', filters.host);
    if (filters.search) params.append('search', filters.search);
    if (filters.search_content) params.append('search_content', filters.search_content);
    if (filters.show_noop) params.append('show_noop', 'true');
    
    const queryString = params.toString();
    const url = queryString ? `/entries?${queryString}` : '/entries';
    
    return this.get<LogEntry[]>(url);
  }

  // Get entry summaries with filters (lightweight, for index page)
  async getEntriesSummary(filters: Filters = {}): Promise<LogEntrySummary[]> {
    const params = new URLSearchParams();
    
    // Resolve date range from preset or explicit dates
    const { date_from, date_to } = this.resolveDateRange(filters);
    
    if (date_from) params.append('date_from', date_from);
    if (date_to) params.append('date_to', date_to);
    if (filters.host) params.append('host', filters.host);
    if (filters.search) params.append('search', filters.search);
    if (filters.search_content) params.append('search_content', filters.search_content);
    if (filters.show_noop) params.append('show_noop', 'true');
    
    const queryString = params.toString();
    const url = queryString ? `/entries/summary?${queryString}` : '/entries/summary';
    
    return this.get<LogEntrySummary[]>(url);
  }

  // Get single entry
  async getEntry(uuid: string): Promise<LogEntry> {
    return this.get<LogEntry>(`/entry/${uuid}`);
  }

  // Update entry
  async updateEntry(data: EntryUpdateRequest): Promise<ApiResponse> {
    return this.post<ApiResponse>('/entry', data);
  }

  // Redact password from single entry
  async redactEntry(data: EntryRedactRequest): Promise<ApiResponse> {
    return this.post<ApiResponse>('/entry/redact', data);
  }

  // Bulk redact passwords
  async bulkRedact(data: BulkRedactRequest): Promise<ApiResponse> {
    return this.post<ApiResponse>('/redact', data);
  }

  // Get diff content
  async getDiffContent(uuid: string): Promise<DiffResponse> {
    // Note: this endpoint doesn't have the /api prefix
    const response = await fetch(`/diffcontent/${uuid}`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  // Get all tasks
  async getTasks(): Promise<Task[]> {
    return this.get<Task[]>('/tasks');
  }

  // Create a new task
  async createTask(data: TaskCreateRequest): Promise<TaskCreateResponse> {
    return this.post<TaskCreateResponse>('/task', data);
  }

  // Update a task
  async updateTask(data: TaskUpdateRequest): Promise<ApiResponse> {
    return this.post<ApiResponse>('/task/update', data);
  }

  // Remove entries from any task
  async ungroupEntries(entryUuids: string[]): Promise<ApiResponse> {
    return this.post<ApiResponse>('/entries/ungroup', entryUuids);
  }

  // Get active task
  async getActiveTask(): Promise<{ task_id: number | null }> {
    return this.get<{ task_id: number | null }>('/active-task');
  }

  // Set active task
  async setActiveTask(taskId: number | null): Promise<ApiResponse> {
    return this.post<ApiResponse>('/active-task', { task_id: taskId });
  }

  // Copy text to clipboard
  async copyToClipboard(text: string): Promise<void> {
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(text);
    } else {
      // Fallback for older browsers
      const textArea = document.createElement('textarea');
      textArea.value = text;
      document.body.appendChild(textArea);
      textArea.select();
      document.execCommand('copy');
      document.body.removeChild(textArea);
    }
  }

  // Generate copy text for entry (works with both LogEntry and LogEntrySummary)
  getCopyText(entry: LogEntry | LogEntrySummary): string {
    if (entry.capture_type === 'Run') {
      return `prodlog run ${entry.cmd}`;
    } else {
      return entry.cmd.startsWith('sudo') 
        ? `prodlog edit -s ${entry.filename}`
        : `prodlog edit ${entry.filename}`;
    }
  }

  // Format timestamp
  formatTimestamp(timestamp: string): string {
    const iso = new Date(timestamp).toISOString(); // "2023-12-25T14:30:45.123Z"
    const [date, time] = iso.split('T');
    const [year, month, day] = date.split('-');
    const [hours, minutes, seconds] = time.split(':');
    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds.split('.')[0]} UTC`;
  }

  // Format duration
  formatDuration(duration_ms: number): string {
    if (duration_ms < 1000) {
      return `${duration_ms}ms`;
    } else if (duration_ms < 60000) {
      return `${(duration_ms / 1000).toFixed(1)}s`;
    } else {
      const minutes = Math.floor(duration_ms / 60000);
      const seconds = ((duration_ms % 60000) / 1000).toFixed(0);
      return `${minutes}m ${seconds}s`;
    }
  }
}

export const api = new ApiService(); 