import type {
  ModelLoadRequest,
  ModelUnloadRequest,
  LogFilters,
} from '../types';

// Custom error class for API errors
export class TabAgentApiError extends Error {
  public status: number;
  public code?: string;
  
  constructor(
    message: string,
    status: number,
    code?: string
  ) {
    super(message);
    this.name = 'TabAgentApiError';
    this.status = status;
    this.code = code;
  }
}

// Request interceptor type
export type RequestInterceptor = (config: RequestInit) => RequestInit | Promise<RequestInit>;

// Response interceptor type
export type ResponseInterceptor = (response: Response) => Response | Promise<Response>;

// API client configuration
export interface ApiClientConfig {
  baseUrl?: string;
  timeout?: number;
  retries?: number;
  retryDelay?: number;
}

/**
 * TabAgent API Client
 * Provides type-safe methods for all TabAgent HTTP endpoints
 */
export class TabAgentApiClient {
  private baseUrl: string;
  private timeout: number;
  private retries: number;
  private retryDelay: number;
  private requestInterceptors: RequestInterceptor[] = [];
  private responseInterceptors: ResponseInterceptor[] = [];

  constructor(config: ApiClientConfig = {}) {
    this.baseUrl = config.baseUrl || 'http://localhost:3000';
    this.timeout = config.timeout || 10000;
    this.retries = config.retries || 3;
    this.retryDelay = config.retryDelay || 1000;

    // Add default request interceptor for logging
    this.addRequestInterceptor((config) => {
      console.log(`[API] ${config.method || 'GET'}`);
      return config;
    });

    // Add default response interceptor for error handling
    this.addResponseInterceptor(async (response) => {
      if (!response.ok) {
        const errorText = await response.text();
        let errorMessage = `HTTP ${response.status}: ${response.statusText}`;
        
        try {
          const errorData = JSON.parse(errorText);
          errorMessage = errorData.error || errorData.message || errorMessage;
        } catch {
          // Use default error message if JSON parsing fails
        }

        throw new TabAgentApiError(errorMessage, response.status);
      }
      return response;
    });
  }

  /**
   * Add request interceptor
   */
  addRequestInterceptor(interceptor: RequestInterceptor): void {
    this.requestInterceptors.push(interceptor);
  }

  /**
   * Add response interceptor
   */
  addResponseInterceptor(interceptor: ResponseInterceptor): void {
    this.responseInterceptors.push(interceptor);
  }

  /**
   * Make HTTP request with retries and interceptors
   */
  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    
    // Apply request interceptors
    let config: RequestInit = {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    };

    for (const interceptor of this.requestInterceptors) {
      config = await interceptor(config);
    }

    let lastError: Error;

    for (let attempt = 0; attempt <= this.retries; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.timeout);

        let response = await fetch(url, {
          ...config,
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        // Apply response interceptors
        for (const interceptor of this.responseInterceptors) {
          response = await interceptor(response);
        }

        // Parse JSON response
        const data = await response.json();
        return data as T;

      } catch (error) {
        lastError = error as Error;
        
        // Don't retry on client errors (4xx) or abort errors
        if (error instanceof TabAgentApiError && error.status >= 400 && error.status < 500) {
          throw error;
        }
        
        if (error instanceof Error && error.name === 'AbortError') {
          throw new TabAgentApiError('Request timeout', 408);
        }

        // Wait before retry (except on last attempt)
        if (attempt < this.retries) {
          await new Promise(resolve => setTimeout(resolve, this.retryDelay * (attempt + 1)));
        }
      }
    }

    throw lastError!;
  }

  // Health and System endpoints
  
  /**
   * Get server health status
   */
  async getHealth(): Promise<any> {
    return this.makeRequest('/v1/health');
  }

  /**
   * Get system information
   */
  async getSystemInfo(): Promise<any> {
    return this.makeRequest('/v1/system/info');
  }

  /**
   * Get performance statistics
   */
  async getStats(): Promise<any> {
    return this.makeRequest('/v1/stats');
  }

  /**
   * Get detailed system statistics
   */
  async getSystemStats(): Promise<any> {
    return this.makeRequest('/v1/system/stats');
  }

  /**
   * Get server status
   */
  async getServerStatus(): Promise<any> {
    return this.makeRequest('/v1/server/status');
  }

  // Model Management endpoints

  /**
   * List all available models
   */
  async listModels(): Promise<any[]> {
    return this.makeRequest('/v1/models');
  }

  /**
   * Get specific model information
   */
  async getModel(modelId: string): Promise<any> {
    return this.makeRequest(`/v1/models/${encodeURIComponent(modelId)}`);
  }

  /**
   * Load a model
   */
  async loadModel(request: ModelLoadRequest): Promise<void> {
    await this.makeRequest('/v1/models/load', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  /**
   * Unload a model
   */
  async unloadModel(request: ModelUnloadRequest): Promise<void> {
    await this.makeRequest('/v1/models/unload', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  /**
   * Get model metrics
   */
  async getModelMetrics(modelId?: string): Promise<Record<string, any>> {
    const endpoint = modelId 
      ? `/v1/models/${encodeURIComponent(modelId)}/metrics`
      : '/v1/models/metrics';
    return this.makeRequest(endpoint);
  }

  // Logging endpoints

  /**
   * Query logs with filters
   */
  async queryLogs(filters: LogFilters, limit = 100, offset = 0): Promise<any> {
    const params = new URLSearchParams({
      limit: limit.toString(),
      offset: offset.toString(),
      ...Object.fromEntries(
        Object.entries(filters).filter(([_, value]) => value !== undefined && value !== '')
      ),
    });

    return this.makeRequest(`/v1/logs?${params}`);
  }

  /**
   * Get log statistics
   */
  async getLogStats(filters?: Partial<LogFilters>): Promise<any> {
    const params = filters 
      ? new URLSearchParams(
          Object.fromEntries(
            Object.entries(filters).filter(([_, value]) => value !== undefined && value !== '')
          )
        )
      : '';

    const endpoint = params ? `/v1/logs/stats?${params}` : '/v1/logs/stats';
    return this.makeRequest(endpoint);
  }

  /**
   * Clear logs
   */
  async clearLogs(filters?: Partial<LogFilters>): Promise<void> {
    await this.makeRequest('/v1/logs/clear', {
      method: 'DELETE',
      body: filters ? JSON.stringify(filters) : undefined,
    });
  }

  // Database endpoints

  /**
   * Get database statistics
   */
  async getDatabaseStats(): Promise<any> {
    return this.makeRequest('/v1/database/stats');
  }

  /**
   * Get database schema information
   */
  async getDatabaseSchema(): Promise<any> {
    return this.makeRequest('/v1/database/schema');
  }

  /**
   * Get database performance metrics
   */
  async getDatabasePerformanceMetrics(): Promise<any> {
    return this.makeRequest('/v1/database/performance');
  }

  /**
   * Search database with advanced query
   */
  async searchDatabase(query: any): Promise<any> {
    return this.makeRequest('/v1/database/search', {
      method: 'POST',
      body: JSON.stringify(query),
    });
  }

  /**
   * Get database nodes with pagination
   */
  async getDatabaseNodes(type?: string, limit = 50, offset = 0): Promise<{
    nodes: any[];
    total: number;
    has_more: boolean;
  }> {
    const params = new URLSearchParams({
      limit: limit.toString(),
      offset: offset.toString(),
    });
    
    if (type) {
      params.append('type', type);
    }

    return this.makeRequest(`/v1/database/nodes?${params}`);
  }

  /**
   * Get specific database node
   */
  async getDatabaseNode(nodeId: string): Promise<any> {
    return this.makeRequest(`/v1/database/nodes/${encodeURIComponent(nodeId)}`);
  }

  /**
   * Get knowledge graph data
   */
  async getKnowledgeGraph(filters?: any): Promise<any> {
    const endpoint = filters 
      ? '/v1/database/graph'
      : '/v1/database/graph';
    
    return this.makeRequest(endpoint, {
      method: filters ? 'POST' : 'GET',
      body: filters ? JSON.stringify(filters) : undefined,
    });
  }

  /**
   * Export database data
   */
  async exportDatabase(options: any): Promise<Blob> {
    const response = await fetch(`${this.baseUrl}/v1/database/export`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(options),
    });

    if (!response.ok) {
      throw new TabAgentApiError(`Export failed: ${response.statusText}`, response.status);
    }

    return response.blob();
  }

  /**
   * Get database backup information
   */
  async getDatabaseBackups(): Promise<any[]> {
    return this.makeRequest('/v1/database/backups');
  }

  /**
   * Create database backup
   */
  async createDatabaseBackup(name: string, type: 'full' | 'incremental' = 'full'): Promise<void> {
    await this.makeRequest('/v1/database/backups', {
      method: 'POST',
      body: JSON.stringify({ name, type }),
    });
  }

  // Configuration endpoints

  /**
   * Get server configuration
   */
  async getConfig(): Promise<Record<string, unknown>> {
    return this.makeRequest('/v1/config');
  }

  /**
   * Update server configuration
   */
  async updateConfig(config: Record<string, unknown>): Promise<void> {
    await this.makeRequest('/v1/config', {
      method: 'PUT',
      body: JSON.stringify(config),
    });
  }

  // Utility methods

  /**
   * Get WebSocket URL for real-time connections
   */
  getWebSocketUrl(path: string): string {
    const wsUrl = this.baseUrl.replace(/^http/, 'ws');
    return `${wsUrl}${path}`;
  }

  /**
   * Test API connectivity
   */
  async testConnection(): Promise<boolean> {
    try {
      await this.getHealth();
      return true;
    } catch {
      return false;
    }
  }
}

// Create default API client instance
export const apiClient = new TabAgentApiClient();

// Export error handling utility
export const handleApiError = (error: unknown): string => {
  if (error instanceof TabAgentApiError) {
    switch (error.status) {
      case 400:
        return 'Bad request - please check your input';
      case 401:
        return 'Unauthorized - authentication required';
      case 403:
        return 'Forbidden - insufficient permissions';
      case 404:
        return 'Resource not found';
      case 408:
        return 'Request timeout - please try again';
      case 429:
        return 'Too many requests - please wait and try again';
      case 500:
        return 'Internal server error - please try again later';
      case 502:
        return 'Bad gateway - server is temporarily unavailable';
      case 503:
        return 'Service unavailable - server is temporarily down';
      case 504:
        return 'Gateway timeout - server took too long to respond';
      default:
        return error.message || 'An API error occurred';
    }
  }
  
  if (error instanceof Error) {
    return error.message;
  }
  
  return 'An unexpected error occurred';
};