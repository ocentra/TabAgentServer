// Database-related types for TabAgent

export interface DatabaseStats {
  nodes: number;
  edges: number;
  size_bytes: number;
  collections: Record<string, number>;
  performance: {
    query_count: number;
    average_query_time: number;
    cache_hit_rate: number;
    index_efficiency: number;
  };
  health: {
    status: 'healthy' | 'degraded' | 'unhealthy';
    last_backup: string;
    disk_usage_percent: number;
    memory_usage_mb: number;
  };
}

export interface DatabaseNode {
  id: string;
  type: 'conversation' | 'message' | 'document' | 'entity' | 'embedding';
  properties: Record<string, any>;
  created_at: string;
  updated_at: string;
  metadata?: Record<string, any>;
}

export interface DatabaseEdge {
  id: string;
  source_id: string;
  target_id: string;
  type: string;
  properties: Record<string, any>;
  weight?: number;
  created_at: string;
}

export interface DatabaseSearchResult {
  id: string;
  type: string;
  title: string;
  content: string;
  score: number;
  metadata: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface DatabaseSearchResponse {
  results: DatabaseSearchResult[];
  total: number;
  query_time_ms: number;
  facets?: Record<string, Record<string, number>>;
}

export interface DatabaseQuery {
  query: string;
  filters?: {
    type?: string[];
    date_range?: {
      start: string;
      end: string;
    };
    properties?: Record<string, any>;
  };
  sort?: {
    field: string;
    order: 'asc' | 'desc';
  };
  limit?: number;
  offset?: number;
}

export interface DatabaseSchema {
  node_types: Array<{
    type: string;
    count: number;
    properties: Array<{
      name: string;
      type: 'string' | 'number' | 'boolean' | 'date' | 'json';
      indexed: boolean;
    }>;
  }>;
  edge_types: Array<{
    type: string;
    count: number;
    source_types: string[];
    target_types: string[];
  }>;
}

export interface DatabasePerformanceMetrics {
  queries_per_second: number;
  average_query_time: number;
  slow_queries: Array<{
    query: string;
    duration_ms: number;
    timestamp: string;
  }>;
  cache_stats: {
    hit_rate: number;
    miss_rate: number;
    evictions: number;
  };
  index_usage: Array<{
    index_name: string;
    usage_count: number;
    efficiency: number;
  }>;
}

export interface KnowledgeGraphNode {
  id: string;
  label: string;
  type: string;
  properties: Record<string, any>;
  x?: number;
  y?: number;
  size?: number;
  color?: string;
}

export interface KnowledgeGraphEdge {
  id: string;
  source: string;
  target: string;
  type: string;
  weight?: number;
  properties?: Record<string, any>;
}

export interface KnowledgeGraphData {
  nodes: KnowledgeGraphNode[];
  edges: KnowledgeGraphEdge[];
  metadata: {
    total_nodes: number;
    total_edges: number;
    node_types: Record<string, number>;
    edge_types: Record<string, number>;
  };
}

export interface DatabaseExportOptions {
  format: 'json' | 'csv' | 'graphml' | 'cypher';
  include_types?: string[];
  date_range?: {
    start: string;
    end: string;
  };
  compression?: boolean;
}

export interface DatabaseBackupInfo {
  id: string;
  name: string;
  size_bytes: number;
  created_at: string;
  status: 'completed' | 'in_progress' | 'failed';
  type: 'full' | 'incremental';
}