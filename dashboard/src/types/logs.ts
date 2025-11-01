// Log entry types
export interface LogEntry {
  id: string;
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  source: string;
  context: string;
  message: string;
  data?: Record<string, unknown>;
}

export interface LogFilters {
  level: string;
  source: string;
  context?: string;
  timeRange: string;
  search?: string;
}

export interface LogQueryResult {
  count: number;
  logs: LogEntry[];
}

export interface LogStats {
  total_logs: number;
  by_level: Record<string, number>;
  by_source: Record<string, number>;
  by_context: Record<string, number>;
  oldest_log?: string;
  newest_log?: string;
}