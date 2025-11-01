import { QueryClient } from '@tanstack/react-query';
import { handleApiError } from './api-client';

/**
 * React Query client configuration
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Cache data for 5 minutes by default
      staleTime: 5 * 60 * 1000,
      // Keep data in cache for 10 minutes
      gcTime: 10 * 60 * 1000,
      // Retry failed requests 3 times with exponential backoff
      retry: (failureCount, error) => {
        // Don't retry on client errors (4xx)
        if (error instanceof Error && 'status' in error) {
          const status = (error as any).status;
          if (status >= 400 && status < 500) {
            return false;
          }
        }
        return failureCount < 3;
      },
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
      // Refetch on window focus for real-time data
      refetchOnWindowFocus: true,
      // Refetch on reconnect
      refetchOnReconnect: true,
    },
    mutations: {
      // Retry mutations once
      retry: 1,
      // Show error notifications for mutations
      onError: (error) => {
        console.error('Mutation error:', handleApiError(error));
      },
    },
  },
});

// Query keys for consistent caching
export const queryKeys = {
  // System queries
  health: ['health'] as const,
  systemInfo: ['system', 'info'] as const,
  systemStats: ['system', 'stats'] as const,
  serverStatus: ['server', 'status'] as const,
  performanceStats: ['performance', 'stats'] as const,
  
  // Model queries
  models: ['models'] as const,
  model: (id: string) => ['models', id] as const,
  modelMetrics: (id?: string) => ['models', 'metrics', id] as const,
  
  // Log queries
  logs: (filters: Record<string, unknown>) => ['logs', filters] as const,
  logStats: (filters?: Record<string, unknown>) => ['logs', 'stats', filters] as const,
  
  // Database queries
  databaseStats: ['database', 'stats'] as const,
  databaseSearch: (query: string) => ['database', 'search', query] as const,
  
  // Config queries
  config: ['config'] as const,
} as const;

// Real-time query options for frequently updated data
export const realTimeQueryOptions = {
  // Update every 5 seconds
  refetchInterval: 5000,
  // Keep refetching even when window is not focused
  refetchIntervalInBackground: true,
  // Consider data stale immediately for real-time updates
  staleTime: 0,
};

// Background query options for less critical data
export const backgroundQueryOptions = {
  // Update every 30 seconds
  refetchInterval: 30000,
  // Don't refetch in background
  refetchIntervalInBackground: false,
  // Keep data fresh for 1 minute
  staleTime: 60000,
};