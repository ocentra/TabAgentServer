// Utility helper functions following n8n's patterns

import { nanoid } from 'nanoid'
import type { CanvasNode, CanvasConnection, WorkflowData } from '@/types'

/**
 * Generate a unique ID for nodes, connections, etc.
 */
export function generateId(): string {
  return nanoid(10)
}

/**
 * Create a new workflow with default settings
 */
export function createNewWorkflow(name: string = 'Untitled Workflow'): WorkflowData {
  return {
    id: generateId(),
    name,
    description: '',
    nodes: [],
    connections: [],
    settings: {
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
      saveDataErrorExecution: 'all',
      saveDataSuccessExecution: 'all',
      saveManualExecutions: true,
      callerPolicy: 'workflowsFromSameOwner'
    },
    createdAt: new Date(),
    updatedAt: new Date(),
    version: 1
  }
}

/**
 * Create a new node with default properties
 */
export function createNode(
  type: string, 
  position: { x: number; y: number },
  name?: string
): CanvasNode {
  return {
    id: generateId(),
    type,
    position,
    data: {
      type,
      name: name || type,
      parameters: {},
      inputs: [],
      outputs: [],
      status: 'idle'
    }
  }
}

/**
 * Create a new connection between nodes
 */
export function createConnection(
  source: string,
  target: string,
  sourceHandle: string = 'output',
  targetHandle: string = 'input'
): CanvasConnection {
  return {
    id: generateId(),
    source,
    target,
    sourceHandle,
    targetHandle,
    type: 'default'
  }
}

/**
 * Validate workflow structure
 */
export function validateWorkflow(workflow: WorkflowData): { valid: boolean; errors: string[] } {
  const errors: string[] = []
  
  // Check for nodes
  if (workflow.nodes.length === 0) {
    errors.push('Workflow must contain at least one node')
  }
  
  // Check for duplicate node IDs
  const nodeIds = workflow.nodes.map(n => n.id)
  const duplicateIds = nodeIds.filter((id, index) => nodeIds.indexOf(id) !== index)
  if (duplicateIds.length > 0) {
    errors.push(`Duplicate node IDs found: ${duplicateIds.join(', ')}`)
  }
  
  // Check connections reference valid nodes
  for (const connection of workflow.connections) {
    if (!nodeIds.includes(connection.source)) {
      errors.push(`Connection references invalid source node: ${connection.source}`)
    }
    if (!nodeIds.includes(connection.target)) {
      errors.push(`Connection references invalid target node: ${connection.target}`)
    }
  }
  
  // Check for circular dependencies (basic check)
  const hasCircularDependency = checkCircularDependency(workflow.nodes, workflow.connections)
  if (hasCircularDependency) {
    errors.push('Workflow contains circular dependencies')
  }
  
  return {
    valid: errors.length === 0,
    errors
  }
}

/**
 * Check for circular dependencies in workflow
 */
function checkCircularDependency(nodes: CanvasNode[], connections: CanvasConnection[]): boolean {
  const graph = new Map<string, string[]>()
  
  // Build adjacency list
  nodes.forEach(node => graph.set(node.id, []))
  connections.forEach(conn => {
    const targets = graph.get(conn.source) || []
    targets.push(conn.target)
    graph.set(conn.source, targets)
  })
  
  // DFS to detect cycles
  const visited = new Set<string>()
  const recursionStack = new Set<string>()
  
  function hasCycle(nodeId: string): boolean {
    if (recursionStack.has(nodeId)) return true
    if (visited.has(nodeId)) return false
    
    visited.add(nodeId)
    recursionStack.add(nodeId)
    
    const neighbors = graph.get(nodeId) || []
    for (const neighbor of neighbors) {
      if (hasCycle(neighbor)) return true
    }
    
    recursionStack.delete(nodeId)
    return false
  }
  
  for (const nodeId of graph.keys()) {
    if (!visited.has(nodeId) && hasCycle(nodeId)) {
      return true
    }
  }
  
  return false
}

/**
 * Format file size in human readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes'
  
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

/**
 * Format duration in human readable format
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`
  return `${(ms / 3600000).toFixed(1)}h`
}

/**
 * Debounce function calls
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null
  
  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

/**
 * Deep clone an object
 */
export function deepClone<T>(obj: T): T {
  if (obj === null || typeof obj !== 'object') return obj
  if (obj instanceof Date) return new Date(obj.getTime()) as unknown as T
  if (obj instanceof Array) return obj.map(item => deepClone(item)) as unknown as T
  if (typeof obj === 'object') {
    const cloned = {} as T
    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        cloned[key] = deepClone(obj[key])
      }
    }
    return cloned
  }
  return obj
}

/**
 * Check if two objects are deeply equal
 */
export function deepEqual(a: any, b: any): boolean {
  if (a === b) return true
  if (a == null || b == null) return false
  if (typeof a !== typeof b) return false
  
  if (typeof a === 'object') {
    const keysA = Object.keys(a)
    const keysB = Object.keys(b)
    
    if (keysA.length !== keysB.length) return false
    
    for (const key of keysA) {
      if (!keysB.includes(key)) return false
      if (!deepEqual(a[key], b[key])) return false
    }
    
    return true
  }
  
  return false
}