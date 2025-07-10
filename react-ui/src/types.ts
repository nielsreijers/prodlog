export interface LogEntry {
  uuid: string;
  capture_type: 'Run' | 'Edit';
  host: string;
  cwd: string;
  cmd: string;
  start_time: string;
  duration_ms: number;
  message: string;
  is_noop: boolean;
  exit_code: number;
  local_user: string;
  remote_user: string;
  captured_output: string; // base64 encoded
  filename: string;
  original_content: string; // base64 encoded
  edited_content: string; // base64 encoded
  terminal_rows: number;
  terminal_cols: number;
  task_id?: number;
}

// Lightweight version for index page - excludes large content fields
export interface LogEntrySummary {
  uuid: string;
  capture_type: 'Run' | 'Edit';
  host: string;
  cwd: string;
  cmd: string;
  start_time: string;
  duration_ms: number;
  message: string;
  is_noop: boolean;
  exit_code: number;
  local_user: string;
  remote_user: string;
  filename: string;
  terminal_rows: number;
  terminal_cols: number;
  task_id?: number;
}

export interface Task {
  id: number;
  name: string;
  created_at: string;
}

export interface TaskGroup {
  task: Task;
  entries: LogEntrySummary[];
  start_time: string;
  end_time: string;
  total_duration: number;
}

export interface Filters {
  date_from?: string;
  date_to?: string;
  host?: string;
  search?: string;
  search_content?: string;
  show_noop?: boolean;
}

export interface ApiResponse<T = any> {
  data?: T;
  error?: string;
  message?: string;
}

export interface BulkRedactRequest {
  passwords: string[];
}

export interface EntryRedactRequest {
  uuid: string;
  password: string;
}

export interface EntryUpdateRequest {
  uuid: string;
  message: string;
  is_noop: boolean;
}

export interface TaskCreateRequest {
  name: string;
  entry_uuids: string[];
}

export interface TaskUpdateRequest {
  task_id: number;
  name?: string;
  entry_uuids?: string[];
}

export interface DiffResponse {
  diff: string;
} 