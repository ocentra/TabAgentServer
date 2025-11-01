// Vue Flow TypeScript type definitions
import type { Connection } from '@vue-flow/core'

// Basic Vue Flow types
export interface CanvasNode {
  id: string
  type: string
  position: { x: number; y: number }
  data: CanvasNodeData
  selected?: boolean
  dragging?: boolean
  dimensions?: { width: number; height: number }
  label?: string
}

export interface CanvasConnection {
  id: string
  source: string
  target: string
  sourceHandle?: string | null
  targetHandle?: string | null
  type?: 'default' | 'smoothstep' | 'step'
  animated?: boolean
  style?: Record<string, any>
}

export interface CanvasNodeData {
  type: string
  name: string
  parameters: Record<string, any>
  position: { x: number; y: number }
  inputs: NodeInput[]
  outputs: NodeOutput[]
  status?: 'idle' | 'running' | 'success' | 'error'
}

export interface NodeInput {
  name: string
  type: string
  required?: boolean
  description?: string
}

export interface NodeOutput {
  name: string
  type: string
  description?: string
}

// Vue Flow viewport and interaction types
export interface ViewportTransform {
  x: number
  y: number
  zoom: number
}

export interface NodeMoveEvent {
  id: string
  position: { x: number; y: number }
  dragging: boolean
}

export interface ConnectionCreateData extends Connection {
  source: string
  target: string
  sourceHandle?: string | null
  targetHandle?: string | null
}

// Vue Flow component props
export interface VueFlowProps {
  nodes: CanvasNode[]
  edges: CanvasConnection[]
  viewport?: ViewportTransform
  pannable?: boolean
  zoomable?: boolean
  selectable?: boolean
  draggable?: boolean
  connectable?: boolean
  snapToGrid?: boolean
  snapGrid?: [number, number]
  onlyRenderVisibleElements?: boolean
  minZoom?: number
  maxZoom?: number
  defaultZoom?: number
  defaultPosition?: [number, number]
  translateExtent?: [[number, number], [number, number]]
  nodeExtent?: [[number, number], [number, number]]
  elevateEdgesOnSelect?: boolean
  elevateNodesOnSelect?: boolean
  fitViewOnInit?: boolean
  nodesConnectable?: boolean
  nodesDraggable?: boolean
  edgesUpdatable?: boolean
  elementsSelectable?: boolean
  selectNodesOnDrag?: boolean
  multiSelectionKeyCode?: string | string[]
  deleteKeyCode?: string | string[]
  selectionKeyCode?: string | string[]
  zoomActivationKeyCode?: string | string[]
  panActivationKeyCode?: string | string[]
}

// Vue Flow events
export interface VueFlowEvents {
  'nodes:change': (changes: any[]) => void
  'edges:change': (changes: any[]) => void
  'node:click': (event: { event: MouseEvent; node: CanvasNode }) => void
  'node:double-click': (event: { event: MouseEvent; node: CanvasNode }) => void
  'node:context-menu': (event: { event: MouseEvent; node: CanvasNode }) => void
  'node:mouse-enter': (event: { event: MouseEvent; node: CanvasNode }) => void
  'node:mouse-leave': (event: { event: MouseEvent; node: CanvasNode }) => void
  'node:mouse-move': (event: { event: MouseEvent; node: CanvasNode }) => void
  'node:drag-start': (event: { event: DragEvent; node: CanvasNode }) => void
  'node:drag': (event: { event: DragEvent; node: CanvasNode }) => void
  'node:drag-stop': (event: { event: DragEvent; node: CanvasNode }) => void
  'edge:click': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'edge:double-click': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'edge:context-menu': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'edge:mouse-enter': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'edge:mouse-leave': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'edge:mouse-move': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'edge:update-start': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'edge:update': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'edge:update-end': (event: { event: MouseEvent; edge: CanvasConnection }) => void
  'connect': (connection: Connection) => void
  'connect:start': (event: { event: MouseEvent; nodeId: string; handleId: string; handleType: string }) => void
  'connect:stop': (event: MouseEvent) => void
  'connect:end': (event: MouseEvent) => void
  'pane:click': (event: MouseEvent) => void
  'pane:context-menu': (event: MouseEvent) => void
  'pane:scroll': (event: WheelEvent) => void
  'selection:drag-start': (event: { event: MouseEvent; nodes: CanvasNode[] }) => void
  'selection:drag': (event: { event: MouseEvent; nodes: CanvasNode[] }) => void
  'selection:drag-stop': (event: { event: MouseEvent; nodes: CanvasNode[] }) => void
  'selection:context-menu': (event: { event: MouseEvent; nodes: CanvasNode[] }) => void
  'viewport:change': (viewport: ViewportTransform) => void
}

// Dagre layout types
export interface DagreLayoutOptions {
  direction: 'TB' | 'BT' | 'LR' | 'RL'
  nodeWidth?: number
  nodeHeight?: number
  rankSep?: number
  nodeSep?: number
  edgeSep?: number
  ranker?: 'network-simplex' | 'tight-tree' | 'longest-path'
}

export interface LayoutedElements {
  nodes: CanvasNode[]
  edges: CanvasConnection[]
}