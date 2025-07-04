import { 
  LogEntry, 
  LogEntrySummary,
  Filters, 
  BulkRedactRequest, 
  EntryRedactRequest, 
  EntryUpdateRequest,
  DiffResponse,
  ApiResponse 
} from './types';

class ApiService {
  private baseUrl = '';

  async get<T>(url: string): Promise<T> {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
  }

  async post<T>(url: string, data: any): Promise<T> {
    const response = await fetch(url, {
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
    
    if (filters.date_from) params.append('date_from', filters.date_from);
    if (filters.date_to) params.append('date_to', filters.date_to);
    if (filters.host) params.append('host', filters.host);
    if (filters.search) params.append('search', filters.search);
    if (filters.search_content) params.append('search_content', filters.search_content);
    if (filters.show_noop) params.append('show_noop', 'true');
    
    const queryString = params.toString();
    const url = queryString ? `/api/entries?${queryString}` : '/api/entries';
    
    return this.get<LogEntry[]>(url);
  }

  // Get entry summaries with filters (lightweight, for index page)
  async getEntriesSummary(filters: Filters = {}): Promise<LogEntrySummary[]> {
    const params = new URLSearchParams();
    
    if (filters.date_from) params.append('date_from', filters.date_from);
    if (filters.date_to) params.append('date_to', filters.date_to);
    if (filters.host) params.append('host', filters.host);
    if (filters.search) params.append('search', filters.search);
    if (filters.search_content) params.append('search_content', filters.search_content);
    if (filters.show_noop) params.append('show_noop', 'true');
    
    const queryString = params.toString();
    const url = queryString ? `/api/entries/summary?${queryString}` : '/api/entries/summary';
    
    return this.get<LogEntrySummary[]>(url);
  }

  // Get single entry
  async getEntry(uuid: string): Promise<LogEntry> {
    return this.get<LogEntry>(`/api/entry/${uuid}`);
  }

  // Update entry
  async updateEntry(data: EntryUpdateRequest): Promise<ApiResponse> {
    return this.post<ApiResponse>('/api/entry', data);
  }

  // Redact password from single entry
  async redactEntry(data: EntryRedactRequest): Promise<ApiResponse> {
    return this.post<ApiResponse>('/api/entry/redact', data);
  }

  // Bulk redact passwords
  async bulkRedact(data: BulkRedactRequest): Promise<ApiResponse> {
    return this.post<ApiResponse>('/api/redact', data);
  }

  // Get diff content
  async getDiffContent(uuid: string): Promise<DiffResponse> {
    return this.get<DiffResponse>(`/diffcontent/${uuid}`);
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
  formatDuration(start_time: string, duration_ms: number): string {
    const start = new Date(start_time);
    const end = new Date(start.getTime() + duration_ms);
    return this.formatTimestamp(end.toISOString());
  }
}

export const api = new ApiService(); 