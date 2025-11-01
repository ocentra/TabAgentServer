import type {
  LogEntry,
  ModelInfo,
  SystemStats,
  PerformanceStats,
} from '../types';

/**
 * Date and time formatting utilities
 */
export const formatters = {
  /**
   * Format timestamp to human-readable string
   */
  timestamp: (timestamp: string | number | Date): string => {
    const date = new Date(timestamp);
    return date.toLocaleString();
  },

  /**
   * Format timestamp to relative time (e.g., "2 minutes ago")
   */
  relativeTime: (timestamp: string | number | Date): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    
    const seconds = Math.floor(diffMs / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);
    
    if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
    if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    return `${seconds} second${seconds > 1 ? 's' : ''} ago`;
  },

  /**
   * Format duration in milliseconds to human-readable string
   */
  duration: (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
    return `${(ms / 3600000).toFixed(1)}h`;
  },

  /**
   * Format bytes to human-readable string
   */
  bytes: (bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  },

  /**
   * Format percentage with specified decimal places
   */
  percentage: (value: number, decimals = 1): string => {
    return `${value.toFixed(decimals)}%`;
  },

  /**
   * Format number with thousands separators
   */
  number: (value: number): string => {
    return value.toLocaleString();
  },

  /**
   * Format tokens per second
   */
  tokensPerSecond: (tps: number): string => {
    if (tps < 1) return `${(tps * 1000).toFixed(0)} tokens/ms`;
    return `${tps.toFixed(1)} tokens/s`;
  },

  /**
   * Format uptime in seconds to human-readable string
   */
  uptime: (seconds: number): string => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    
    const parts = [];
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    if (minutes > 0) parts.push(`${minutes}m`);
    if (secs > 0 || parts.length === 0) parts.push(`${secs}s`);
    
    return parts.join(' ');
  },
};

/**
 * Data transformation utilities
 */
export const transformers = {
  /**
   * Transform log entry for display
   */
  logEntry: (log: LogEntry) => ({
    ...log,
    formattedTimestamp: formatters.timestamp(log.timestamp),
    relativeTime: formatters.relativeTime(log.timestamp),
    levelColor: getLevelColor(log.level),
    levelIcon: getLevelIcon(log.level),
  }),

  /**
   * Transform model info for display
   */
  modelInfo: (model: ModelInfo) => ({
    ...model,
    formattedMemoryUsage: model.memory_usage ? formatters.bytes(model.memory_usage) : 'N/A',
    formattedParameters: model.parameters ? formatters.number(model.parameters) : 'N/A',
    statusColor: getModelStatusColor(model.status),
    statusIcon: getModelStatusIcon(model.status),
    loadedTime: model.loaded_at ? formatters.relativeTime(model.loaded_at) : null,
  }),

  /**
   * Transform system stats for display
   */
  systemStats: (stats: SystemStats) => ({
    ...stats,
    cpu: {
      ...stats.cpu,
      formattedUsage: formatters.percentage(stats.cpu.usage_percent),
      formattedFrequency: `${(stats.cpu.frequency / 1000).toFixed(1)} GHz`,
    },
    memory: {
      ...stats.memory,
      formattedTotal: formatters.bytes(stats.memory.total),
      formattedUsed: formatters.bytes(stats.memory.used),
      formattedAvailable: formatters.bytes(stats.memory.available),
      formattedUsage: formatters.percentage(stats.memory.usage_percent),
    },
    gpu: stats.gpu ? {
      ...stats.gpu,
      formattedMemoryTotal: formatters.bytes(stats.gpu.memory_total),
      formattedMemoryUsed: formatters.bytes(stats.gpu.memory_used),
      formattedUsage: formatters.percentage(stats.gpu.usage_percent),
      formattedTemperature: stats.gpu.temperature ? `${stats.gpu.temperature}°C` : 'N/A',
    } : null,
    disk: {
      ...stats.disk,
      formattedTotal: formatters.bytes(stats.disk.total),
      formattedUsed: formatters.bytes(stats.disk.used),
      formattedAvailable: formatters.bytes(stats.disk.available),
      formattedUsage: formatters.percentage(stats.disk.usage_percent),
    },
    network: {
      ...stats.network,
      formattedBytesSent: formatters.bytes(stats.network.bytes_sent),
      formattedBytesReceived: formatters.bytes(stats.network.bytes_received),
      formattedConnections: formatters.number(stats.network.connections),
    },
  }),

  /**
   * Transform performance stats for display
   */
  performanceStats: (stats: PerformanceStats) => ({
    ...stats,
    formattedRps: formatters.number(stats.requests_per_second),
    formattedResponseTime: formatters.duration(stats.average_response_time),
    formattedErrorRate: formatters.percentage(stats.error_rate),
    inference_stats: {
      ...stats.inference_stats,
      formattedTotalInferences: formatters.number(stats.inference_stats.total_inferences),
      formattedTtft: formatters.duration(stats.inference_stats.average_ttft),
      formattedTps: formatters.tokensPerSecond(stats.inference_stats.average_tokens_per_second),
    },
  }),
};

/**
 * Data normalization utilities
 */
export const normalizers = {
  /**
   * Normalize log level to consistent format
   */
  logLevel: (level: string): 'debug' | 'info' | 'warn' | 'error' => {
    const normalized = level.toLowerCase();
    if (['debug', 'info', 'warn', 'error'].includes(normalized)) {
      return normalized as 'debug' | 'info' | 'warn' | 'error';
    }
    return 'info'; // Default fallback
  },

  /**
   * Normalize model status to consistent format
   */
  modelStatus: (status: string): 'loaded' | 'loading' | 'unloaded' | 'error' => {
    const normalized = status.toLowerCase();
    if (['loaded', 'loading', 'unloaded', 'error'].includes(normalized)) {
      return normalized as 'loaded' | 'loading' | 'unloaded' | 'error';
    }
    return 'unloaded'; // Default fallback
  },

  /**
   * Normalize timestamp to ISO string
   */
  timestamp: (timestamp: string | number | Date): string => {
    return new Date(timestamp).toISOString();
  },

  /**
   * Normalize percentage value (0-100)
   */
  percentage: (value: number): number => {
    return Math.max(0, Math.min(100, value));
  },

  /**
   * Normalize bytes value (ensure non-negative)
   */
  bytes: (value: number): number => {
    return Math.max(0, value);
  },
};

/**
 * Export utilities for CSV/JSON
 */
export const exporters = {
  /**
   * Convert data to CSV format
   */
  toCSV: <T extends Record<string, unknown>>(data: T[], headers?: string[]): string => {
    if (data.length === 0) return '';
    
    const keys = headers || Object.keys(data[0]);
    const csvHeaders = keys.join(',');
    
    const csvRows = data.map(row => 
      keys.map(key => {
        const value = row[key];
        // Escape commas and quotes in CSV
        if (typeof value === 'string' && (value.includes(',') || value.includes('"'))) {
          return `"${value.replace(/"/g, '""')}"`;
        }
        return String(value ?? '');
      }).join(',')
    );
    
    return [csvHeaders, ...csvRows].join('\n');
  },

  /**
   * Convert data to JSON format with pretty printing
   */
  toJSON: <T>(data: T): string => {
    return JSON.stringify(data, null, 2);
  },

  /**
   * Download data as file
   */
  downloadFile: (content: string, filename: string, mimeType = 'text/plain'): void => {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    
    URL.revokeObjectURL(url);
  },

  /**
   * Export logs to CSV
   */
  exportLogs: (logs: LogEntry[], filename = 'logs.csv'): void => {
    const transformedLogs = logs.map(transformers.logEntry);
    const csv = exporters.toCSV(transformedLogs, [
      'timestamp', 'level', 'source', 'context', 'message'
    ]);
    exporters.downloadFile(csv, filename, 'text/csv');
  },

  /**
   * Export system stats to JSON
   */
  exportSystemStats: (stats: SystemStats, filename = 'system-stats.json'): void => {
    const json = exporters.toJSON(transformers.systemStats(stats));
    exporters.downloadFile(json, filename, 'application/json');
  },
};

// Helper functions for UI styling

function getLevelColor(level: string): string {
  switch (level.toLowerCase()) {
    case 'debug': return 'text-gray-500';
    case 'info': return 'text-blue-500';
    case 'warn': return 'text-yellow-500';
    case 'error': return 'text-red-500';
    default: return 'text-gray-500';
  }
}

function getLevelIcon(level: string): string {
  switch (level.toLowerCase()) {
    case 'debug': return '🐛';
    case 'info': return 'ℹ️';
    case 'warn': return '⚠️';
    case 'error': return '❌';
    default: return 'ℹ️';
  }
}

function getModelStatusColor(status: string): string {
  switch (status.toLowerCase()) {
    case 'loaded': return 'text-green-500';
    case 'loading': return 'text-yellow-500';
    case 'unloaded': return 'text-gray-500';
    case 'error': return 'text-red-500';
    default: return 'text-gray-500';
  }
}

function getModelStatusIcon(status: string): string {
  switch (status.toLowerCase()) {
    case 'loaded': return '✅';
    case 'loading': return '⏳';
    case 'unloaded': return '⭕';
    case 'error': return '❌';
    default: return '⭕';
  }
}