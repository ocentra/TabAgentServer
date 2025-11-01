import type { Node, XYPosition } from '@vue-flow/core'

// Base workflow types following n8n's structure
export interface WorkflowData {
  id: string
  name: string
  description?: string
  nodes: CanvasNode[]
  connections: CanvasConnection[]
  settings: WorkflowSettings
  createdAt: Date
  updatedAt: Date
  version: number
}

export interface WorkflowSettings {
  timezone: string
  saveDataErrorExecution: 'all' | 'none'
  saveDataSuccessExecution: 'all' | 'none'
  saveManualExecutions: boolean
  callerPolicy: 'workflowsFromSameOwner' | 'workflowsFromAList' | 'any'
}

// Canvas node types
export interface CanvasNode extends Node {
  id: string
  type: string
  position: XYPosition
  data: CanvasNodeData
  selected?: boolean
  dragging?: boolean
  dimensions?: { width: number; height: number }
}

export interface CanvasNodeData {
  type: string
  name: string
  parameters: Record<string, any>
  inputs: NodeInput[]
  outputs: NodeOutput[]
  status?: NodeExecutionStatus
  disabled?: boolean
  notes?: string
}

export interface NodeInput {
  type: string
  displayName: string
  required?: boolean
}

export interface NodeOutput {
  type: string
  displayName: string
}

export type NodeExecutionStatus = 'idle' | 'running' | 'success' | 'error' | 'warning'

// Connection types
export interface CanvasConnection {
  id: string
  source: string
  target: string
  sourceHandle: string
  targetHandle: string
  type?: 'default' | 'smoothstep' | 'step'
  animated?: boolean
  style?: Record<string, any>
}

// Node type definitions
export interface NodeTypeDefinition {
  name: string
  displayName: string
  category: NodeCategory
  description: string
  icon: string
  color: string
  inputs: NodeInputDefinition[]
  outputs: NodeOutputDefinition[]
  parameters: NodeParameterDefinition[]
  credentials?: CredentialDefinition[]
}

export enum NodeCategory {
  AI_MODELS = 'ai-models',
  DATA_CONNECTORS = 'data-connectors',
  LOGIC = 'logic',
  TRIGGERS = 'triggers',
  ACTIONS = 'actions',
  UTILITIES = 'utilities'
}

export interface NodeInputDefinition {
  name: string
  type: string
  displayName: string
  required?: boolean
  description?: string
}

export interface NodeOutputDefinition {
  name: string
  type: string
  displayName: string
  description?: string
}

export interface NodeParameterDefinition {
  name: string
  type: 'string' | 'number' | 'boolean' | 'options' | 'json' | 'code'
  displayName: string
  required?: boolean
  default?: any
  description?: string
  options?: Array<{ name: string; value: any }>
}

export interface CredentialDefinition {
  name: string
  displayName: string
  required: boolean
}

// Execution types
export interface ExecutionData {
  id: string
  workflowId: string
  status: ExecutionStatus
  startedAt: Date
  finishedAt?: Date
  mode: ExecutionMode
  data: ExecutionNodeData[]
}

export type ExecutionStatus = 'new' | 'running' | 'success' | 'error' | 'canceled'
export type ExecutionMode = 'manual' | 'trigger' | 'webhook'

export interface ExecutionNodeData {
  nodeId: string
  status: NodeExecutionStatus
  startTime?: Date
  endTime?: Date
  data?: any[]
  error?: ExecutionError
}

export interface ExecutionError {
  message: string
  stack?: string
  type: string
}

// Clipboard types
export interface ClipboardData {
  nodes: CanvasNode[]
  connections: CanvasConnection[]
}