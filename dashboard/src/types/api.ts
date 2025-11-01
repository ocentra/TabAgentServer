// API response types
export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  timestamp: string;
  services: {
    http: boolean;
    webrtc: boolean;
    native_messaging: boolean;
  };
}

export interface SystemInfo {
  version: string;
  uptime: number;
  cpu_usage: number;
  memory_usage: number;
  gpu_usage?: number;
  disk_usage: number;
  active_connections: number;
}

export interface PerformanceStats {
  requests_per_second: number;
  average_response_time: number;
  error_rate: number;
  inference_stats: {
    total_inferences: number;
    average_ttft: number;
    average_tokens_per_second: number;
  };
}

// Generic API response wrapper
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: string;
}

// API error types
export interface ApiError {
  message: string;
  status: number;
  code?: string;
}