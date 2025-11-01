import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api-client';
import type { 
  DatabaseStats, 
  DatabaseQuery, 
  DatabaseSearchResponse,
  DatabaseSchema,
  DatabasePerformanceMetrics,
  KnowledgeGraphData,
  DatabaseExportOptions,
  DatabaseBackupInfo
} from '@/types/database';

// Query keys for consistent caching
export const databaseKeys = {
  all: ['database'] as const,
  stats: () => [...databaseKeys.all, 'stats'] as const,
  schema: () => [...databaseKeys.all, 'schema'] as const,
  performance: () => [...databaseKeys.all, 'performance'] as const,
  nodes: (type?: string, limit?: number, offset?: number) => 
    [...databaseKeys.all, 'nodes', { type, limit, offset }] as const,
  node: (id: string) => [...databaseKeys.all, 'node', id] as const,
  search: (query: DatabaseQuery) => [...databaseKeys.all, 'search', query] as const,
  graph: (filters?: any) => [...databaseKeys.all, 'graph', filters] as const,
  backups: () => [...databaseKeys.all, 'backups'] as const,
};

/**
 * Hook to fetch database statistics
 */
export const useDatabaseStats = () => {
  return useQuery({
    queryKey: databaseKeys.stats(),
    queryFn: () => apiClient.getDatabaseStats() as Promise<DatabaseStats>,
    refetchInterval: 30000, // Refresh every 30 seconds
    staleTime: 15000, // Consider data stale after 15 seconds
  });
};

/**
 * Hook to fetch database schema information
 */
export const useDatabaseSchema = () => {
  return useQuery({
    queryKey: databaseKeys.schema(),
    queryFn: () => apiClient.getDatabaseSchema() as Promise<DatabaseSchema>,
    staleTime: 5 * 60 * 1000, // Schema changes infrequently, cache for 5 minutes
  });
};

/**
 * Hook to fetch database performance metrics
 */
export const useDatabasePerformance = () => {
  return useQuery({
    queryKey: databaseKeys.performance(),
    queryFn: () => apiClient.getDatabasePerformanceMetrics() as Promise<DatabasePerformanceMetrics>,
    refetchInterval: 10000, // Refresh every 10 seconds
    staleTime: 5000,
  });
};

/**
 * Hook to fetch database nodes with pagination
 */
export const useDatabaseNodes = (type?: string, limit = 50, offset = 0) => {
  return useQuery({
    queryKey: databaseKeys.nodes(type, limit, offset),
    queryFn: () => apiClient.getDatabaseNodes(type, limit, offset),
    keepPreviousData: true, // Keep previous data while loading new pages
  });
};

/**
 * Hook to fetch a specific database node
 */
export const useDatabaseNode = (nodeId: string) => {
  return useQuery({
    queryKey: databaseKeys.node(nodeId),
    queryFn: () => apiClient.getDatabaseNode(nodeId),
    enabled: !!nodeId, // Only run query if nodeId is provided
  });
};

/**
 * Hook to search database
 */
export const useDatabaseSearch = (query: DatabaseQuery) => {
  return useQuery({
    queryKey: databaseKeys.search(query),
    queryFn: () => apiClient.searchDatabase(query) as Promise<DatabaseSearchResponse>,
    enabled: !!query.query, // Only search if query is provided
    keepPreviousData: true,
  });
};

/**
 * Hook to fetch knowledge graph data
 */
export const useKnowledgeGraph = (filters?: any) => {
  return useQuery({
    queryKey: databaseKeys.graph(filters),
    queryFn: () => apiClient.getKnowledgeGraph(filters) as Promise<KnowledgeGraphData>,
    staleTime: 2 * 60 * 1000, // Cache for 2 minutes
  });
};

/**
 * Hook to fetch database backups
 */
export const useDatabaseBackups = () => {
  return useQuery({
    queryKey: databaseKeys.backups(),
    queryFn: () => apiClient.getDatabaseBackups() as Promise<DatabaseBackupInfo[]>,
    staleTime: 60 * 1000, // Cache for 1 minute
  });
};

/**
 * Mutation hook for database search
 */
export const useSearchDatabase = () => {
  return useMutation({
    mutationFn: (query: DatabaseQuery) => 
      apiClient.searchDatabase(query) as Promise<DatabaseSearchResponse>,
  });
};

/**
 * Mutation hook for database export
 */
export const useExportDatabase = () => {
  return useMutation({
    mutationFn: (options: DatabaseExportOptions) => 
      apiClient.exportDatabase(options),
  });
};

/**
 * Mutation hook for creating database backup
 */
export const useCreateBackup = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ name, type }: { name: string; type: 'full' | 'incremental' }) =>
      apiClient.createDatabaseBackup(name, type),
    onSuccess: () => {
      // Invalidate backups query to refresh the list
      queryClient.invalidateQueries({ queryKey: databaseKeys.backups() });
    },
  });
};

/**
 * Hook to invalidate all database queries (useful for manual refresh)
 */
export const useRefreshDatabase = () => {
  const queryClient = useQueryClient();
  
  return () => {
    queryClient.invalidateQueries({ queryKey: databaseKeys.all });
  };
};