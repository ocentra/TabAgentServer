// API types for TabAgent server integration
export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: string
  message?: string
}

// Workflow API types
export interface GetWorkflowsResponse {
  workflows: WorkflowSummary[]
  total: number
  page: number
  limit: number
}

export interface WorkflowSummary {
  id: string
  name: string
  description?: string
  createdAt: string
  updatedAt: string
  status: 'active' | 'inactive'
  nodeCount: number
}

export interface CreateWorkflowRequest {
  name: string
  description?: string
  nodes: WorkflowNode[]
  connections: WorkflowConnection[]
}

export interface UpdateWorkflowRequest {
  name?: string
  description?: string
  nodes?: WorkflowNode[]
  connections?: WorkflowConnection[]
}

export interface WorkflowNode {
  id: string
  type: string
  position: { x: number; y: number }
  data: Record<string, any>
}

export interface WorkflowConnection {
  id: string
  source: string
  target: string
  sourceHandle: string
  targetHandle: string
}

// Node Types API
export interface GetNodeTypesResponse {
  node_types: RustNodeTypeDefinition[]
  categories: string[]
  available_models: string[]
  available_connectors: ConnectorInfo[]
}

export interface RustNodeTypeDefinition {
  id: string
  name: string
  category: string
  description: string
  icon: string
  color: string
  inputs: Array<{ name: string; type: string; displayName: string; required?: boolean }>
  outputs: Array<{ name: string; type: string; displayName: string }>
  parameters: RustParameterDefinition[]
  rust_implementation: string
  execution_timeout?: number
  resource_requirements?: ResourceRequirements
}

export interface RustParameterDefinition {
  name: string
  type: string
  display_name: string
  required?: boolean
  default?: any
  description?: string
  options?: Array<{ name: string; value: any }>
}

export interface ResourceRequirements {
  memory_mb?: number
  cpu_cores?: number
  gpu_required?: boolean
}

export interface ConnectorInfo {
  name: string
  display_name: string
  auth_type: 'oauth2' | 'api-key' | 'basic'
  operations: string[]
}

// Execution API types
export interface ExecuteWorkflowRequest {
  mode: 'manual' | 'trigger' | 'test'
  input_data?: Record<string, any>
  debug_mode?: boolean
}

export interface ExecuteWorkflowResponse {
  execution_id: string
  status: 'queued' | 'running'
  estimated_duration?: number
}

export interface GetExecutionResponse {
  execution: RustExecutionData
  node_results: NodeExecutionResult[]
  logs: ExecutionLog[]
  performance_metrics: ExecutionMetrics
}

export interface RustExecutionData {
  id: string
  workflow_id: string
  status: string
  started_at: string
  finished_at?: string
  mode: string
}

export interface NodeExecutionResult {
  node_id: string
  status: string
  start_time?: string
  end_time?: string
  data?: any
  error?: RustExecutionError
}

export interface RustExecutionError {
  message: string
  error_type: string
  stack?: string
}

export interface ExecutionLog {
  timestamp: string
  level: 'debug' | 'info' | 'warn' | 'error'
  message: string
  node_id?: string
}

export interface ExecutionMetrics {
  total_duration_ms: number
  node_execution_times: Record<string, number>
  memory_usage_mb: number
  cpu_usage_percent: number
}

// WebSocket message types
export interface RustExecutionUpdate {
  execution_id: string
  node_id: string
  status: 'waiting' | 'running' | 'success' | 'error' | 'skipped'
  progress?: number
  data?: any
  error?: RustExecutionError
  timestamp: string
}