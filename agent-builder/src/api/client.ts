// API client for TabAgent server integration

import type { 
  ApiResponse, 
  GetWorkflowsResponse, 
  CreateWorkflowRequest, 
  UpdateWorkflowRequest,
  GetNodeTypesResponse,
  ExecuteWorkflowRequest,
  ExecuteWorkflowResponse,
  GetExecutionResponse
} from '@/types'

class ApiClient {
  private baseURL: string
  
  constructor(baseURL: string = 'http://localhost:3000') {
    this.baseURL = baseURL
  }
  
  private async request<T>(
    endpoint: string, 
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseURL}${endpoint}`
    
    const defaultOptions: RequestInit = {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers
      }
    }
    
    try {
      const response = await fetch(url, { ...defaultOptions, ...options })
      const data = await response.json()
      
      if (!response.ok) {
        return {
          success: false,
          error: data.error || `HTTP ${response.status}: ${response.statusText}`
        }
      }
      
      return {
        success: true,
        data
      }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error'
      }
    }
  }
  
  // Workflow API methods
  async getWorkflows(page = 1, limit = 50): Promise<ApiResponse<GetWorkflowsResponse>> {
    return this.request(`/v1/agent-builder/workflows?page=${page}&limit=${limit}`)
  }
  
  async getWorkflow(id: string): Promise<ApiResponse<any>> {
    return this.request(`/v1/agent-builder/workflows/${id}`)
  }
  
  async createWorkflow(workflow: CreateWorkflowRequest): Promise<ApiResponse<any>> {
    return this.request('/v1/agent-builder/workflows', {
      method: 'POST',
      body: JSON.stringify(workflow)
    })
  }
  
  async updateWorkflow(id: string, workflow: UpdateWorkflowRequest): Promise<ApiResponse<any>> {
    return this.request(`/v1/agent-builder/workflows/${id}`, {
      method: 'PUT',
      body: JSON.stringify(workflow)
    })
  }
  
  async deleteWorkflow(id: string): Promise<ApiResponse<void>> {
    return this.request(`/v1/agent-builder/workflows/${id}`, {
      method: 'DELETE'
    })
  }
  
  async validateWorkflow(id: string): Promise<ApiResponse<any>> {
    return this.request(`/v1/agent-builder/workflows/${id}/validate`, {
      method: 'POST'
    })
  }
  
  // Node Types API methods
  async getNodeTypes(): Promise<ApiResponse<GetNodeTypesResponse>> {
    return this.request('/v1/agent-builder/node-types')
  }
  
  async getNodeType(type: string): Promise<ApiResponse<any>> {
    return this.request(`/v1/agent-builder/node-types/${type}`)
  }
  
  async getConnectors(): Promise<ApiResponse<any>> {
    return this.request('/v1/agent-builder/connectors')
  }
  
  // Execution API methods
  async executeWorkflow(id: string, request: ExecuteWorkflowRequest): Promise<ApiResponse<ExecuteWorkflowResponse>> {
    return this.request(`/v1/agent-builder/workflows/${id}/execute`, {
      method: 'POST',
      body: JSON.stringify(request)
    })
  }
  
  async getExecution(id: string): Promise<ApiResponse<GetExecutionResponse>> {
    return this.request(`/v1/agent-builder/executions/${id}`)
  }
  
  async stopExecution(id: string): Promise<ApiResponse<any>> {
    return this.request(`/v1/agent-builder/executions/${id}/stop`, {
      method: 'POST'
    })
  }
  
  // WebSocket connection for real-time updates
  createWebSocket(executionId?: string): WebSocket {
    const wsUrl = `ws://localhost:3000/ws/agent-builder/executions${executionId ? `?execution_id=${executionId}` : ''}`
    return new WebSocket(wsUrl)
  }
}

// Export singleton instance
export const apiClient = new ApiClient()
export default apiClient