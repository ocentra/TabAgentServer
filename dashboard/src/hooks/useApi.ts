import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import { queryKeys, realTimeQueryOptions, backgroundQueryOptions } from '../lib/query-client';
import type {
  ModelLoadRequest,
  ModelUnloadRequest,
  LogFilters,
} from '../types';

// System hooks

/**
 * Hook for fetching server health status
 */
export const useHealth = () => {
  return useQuery({
    queryKey: queryKeys.health,
    queryFn: () => apiClient.getHealth(),
    ...realTimeQueryOptions,
  });
};

/**
 * Hook for fetching system information
 */
export const useSystemInfo = () => {
  return useQuery({
    queryKey: queryKeys.systemInfo,
    queryFn: () => apiClient.getSystemInfo(),
    ...backgroundQueryOptions,
  });
};

/**
 * Hook for fetching detailed system statistics
 */
export const useSystemStats = () => {
  return useQuery({
    queryKey: queryKeys.systemStats,
    queryFn: () => apiClient.getSystemStats(),
    ...realTimeQueryOptions,
  });
};

/**
 * Hook for fetching server status
 */
export const useServerStatus = () => {
  return useQuery({
    queryKey: queryKeys.serverStatus,
    queryFn: () => apiClient.getServerStatus(),
    ...realTimeQueryOptions,
  });
};

/**
 * Hook for fetching performance statistics
 */
export const usePerformanceStats = () => {
  return useQuery({
    queryKey: queryKeys.performanceStats,
    queryFn: () => apiClient.getStats(),
    ...realTimeQueryOptions,
  });
};

// Model hooks

/**
 * Hook for fetching all models
 */
export const useModels = () => {
  return useQuery({
    queryKey: queryKeys.models,
    queryFn: () => apiClient.listModels(),
    ...backgroundQueryOptions,
  });
};

/**
 * Hook for fetching a specific model
 */
export const useModel = (modelId: string) => {
  return useQuery({
    queryKey: queryKeys.model(modelId),
    queryFn: () => apiClient.getModel(modelId),
    enabled: !!modelId,
    ...backgroundQueryOptions,
  });
};

/**
 * Hook for fetching model metrics
 */
export const useModelMetrics = (modelId?: string) => {
  return useQuery({
    queryKey: queryKeys.modelMetrics(modelId),
    queryFn: () => apiClient.getModelMetrics(modelId),
    ...realTimeQueryOptions,
  });
};

/**
 * Hook for loading a model
 */
export const useLoadModel = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (request: ModelLoadRequest) => apiClient.loadModel(request),
    onSuccess: () => {
      // Invalidate and refetch models data
      queryClient.invalidateQueries({ queryKey: queryKeys.models });
      queryClient.invalidateQueries({ queryKey: queryKeys.modelMetrics() });
    },
    onError: (error) => {
      console.error('Failed to load model:', error);
    },
  });
};

/**
 * Hook for unloading a model
 */
export const useUnloadModel = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (request: ModelUnloadRequest) => apiClient.unloadModel(request),
    onSuccess: () => {
      // Invalidate and refetch models data
      queryClient.invalidateQueries({ queryKey: queryKeys.models });
      queryClient.invalidateQueries({ queryKey: queryKeys.modelMetrics() });
    },
    onError: (error) => {
      console.error('Failed to unload model:', error);
    },
  });
};

// Log hooks

/**
 * Hook for querying logs with filters
 */
export const useLogs = (filters: LogFilters, limit = 100, offset = 0) => {
  return useQuery({
    queryKey: queryKeys.logs({ ...filters, limit, offset }),
    queryFn: () => apiClient.queryLogs(filters, limit, offset),
    // Don't auto-refetch logs as they can be large
    refetchInterval: false,
    refetchOnWindowFocus: false,
    // Keep logs data for longer
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
};

/**
 * Hook for fetching log statistics
 */
export const useLogStats = (filters?: Partial<LogFilters>) => {
  return useQuery({
    queryKey: queryKeys.logStats(filters),
    queryFn: () => apiClient.getLogStats(filters),
    ...backgroundQueryOptions,
  });
};

/**
 * Hook for clearing logs
 */
export const useClearLogs = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (filters?: Partial<LogFilters>) => apiClient.clearLogs(filters),
    onSuccess: () => {
      // Invalidate all log-related queries
      queryClient.invalidateQueries({ queryKey: ['logs'] });
    },
    onError: (error) => {
      console.error('Failed to clear logs:', error);
    },
  });
};

// Database hooks

/**
 * Hook for fetching database statistics
 */
export const useDatabaseStats = () => {
  return useQuery({
    queryKey: queryKeys.databaseStats,
    queryFn: () => apiClient.getDatabaseStats(),
    ...backgroundQueryOptions,
  });
};

/**
 * Hook for searching database
 */
export const useDatabaseSearch = (query: string, enabled = true) => {
  return useQuery({
    queryKey: queryKeys.databaseSearch(query),
    queryFn: () => apiClient.searchDatabase(query),
    enabled: enabled && !!query.trim(),
    // Don't auto-refetch search results
    refetchInterval: false,
    refetchOnWindowFocus: false,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

// Configuration hooks

/**
 * Hook for fetching server configuration
 */
export const useConfig = () => {
  return useQuery({
    queryKey: queryKeys.config,
    queryFn: () => apiClient.getConfig(),
    ...backgroundQueryOptions,
  });
};

/**
 * Hook for updating server configuration
 */
export const useUpdateConfig = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (config: Record<string, unknown>) => apiClient.updateConfig(config),
    onSuccess: () => {
      // Invalidate config query to refetch updated data
      queryClient.invalidateQueries({ queryKey: queryKeys.config });
    },
    onError: (error) => {
      console.error('Failed to update config:', error);
    },
  });
};

// Utility hooks

/**
 * Hook for testing API connectivity
 */
export const useApiConnection = () => {
  return useQuery({
    queryKey: ['api', 'connection'],
    queryFn: () => apiClient.testConnection(),
    retry: false,
    refetchInterval: 30000, // Check every 30 seconds
    refetchOnWindowFocus: true,
  });
};

/**
 * Hook for prefetching data
 */
export const usePrefetchData = () => {
  const queryClient = useQueryClient();
  
  const prefetchSystemData = () => {
    queryClient.prefetchQuery({
      queryKey: queryKeys.health,
      queryFn: () => apiClient.getHealth(),
    });
    
    queryClient.prefetchQuery({
      queryKey: queryKeys.systemStats,
      queryFn: () => apiClient.getSystemStats(),
    });
    
    queryClient.prefetchQuery({
      queryKey: queryKeys.models,
      queryFn: () => apiClient.listModels(),
    });
  };
  
  return { prefetchSystemData };
};