// Vue Flow configuration and setup utilities
import type { VueFlowProps } from '@/types/vue-flow'

// Default Vue Flow configuration for canvas
export const defaultVueFlowConfig: Partial<VueFlowProps> = {
  // Canvas behavior
  pannable: true,
  zoomable: true,
  selectable: true,
  draggable: true,
  connectable: true,
  
  // Grid and snapping
  snapToGrid: false,
  snapGrid: [15, 15],
  
  // Performance
  onlyRenderVisibleElements: true,
  
  // Zoom settings
  minZoom: 0.1,
  maxZoom: 4,
  defaultZoom: 1,
  
  // Initial view
  fitViewOnInit: true,
  defaultPosition: [0, 0],
  
  // Node and edge behavior
  nodesConnectable: true,
  nodesDraggable: true,
  edgesUpdatable: true,
  elementsSelectable: true,
  selectNodesOnDrag: false, // FALSE = smoother dragging!
  elevateEdgesOnSelect: true,
  elevateNodesOnSelect: true,
  
  // Smooth dragging (STOLEN from n8n!)
  autoPanOnNodeDrag: true, // Auto-pan when dragging near edges
  autoPanOnConnect: true,  // Auto-pan when connecting
  panOnDrag: true,         // Allow canvas panning while dragging
  
  // Performance for smooth dragging
  edgeUpdaterRadius: 10,   // Makes edge updates smoother
  connectionRadius: 20,    // Larger connection area
  
  // Keyboard shortcuts
  multiSelectionKeyCode: 'Meta', // Cmd on Mac, Ctrl on Windows
  deleteKeyCode: 'Delete',
  selectionKeyCode: 'Shift',
  zoomActivationKeyCode: 'Meta',
  panActivationKeyCode: 'Space'
}

// Vue Flow theme configuration
export const vueFlowTheme = {
  // Node colors
  nodeColors: {
    default: '#ffffff',
    selected: '#ff6d5a',
    hover: '#f0f0f0',
    error: '#ff4757',
    success: '#2ed573',
    warning: '#ffa502',
    running: '#3742fa'
  },
  
  // Edge colors
  edgeColors: {
    default: '#b1b1b7',
    selected: '#ff6d5a',
    hover: '#888888',
    animated: '#ff6d5a'
  },
  
  // Background
  background: {
    color: '#f9f9f9',
    patternColor: '#e1e1e1'
  },
  
  // Controls
  controls: {
    background: '#ffffff',
    border: '#e1e1e1',
    color: '#666666'
  }
}

// CSS custom properties for Vue Flow theming
export const vueFlowCSSVariables = {
  '--vf-node-bg': vueFlowTheme.nodeColors.default,
  '--vf-node-border': '#e1e1e1',
  '--vf-node-color': '#333333',
  '--vf-connection-path': vueFlowTheme.edgeColors.default,
  '--vf-handle': '#555555',
  '--vf-handle-border': '#ffffff',
  '--vf-selection-outline': vueFlowTheme.nodeColors.selected,
  '--vf-box-shadow': '0 1px 4px rgba(0, 0, 0, 0.08)',
  '--vf-edge-stroke-width': '2px',
  '--vf-edge-stroke-selected-width': '3px'
}

// Apply Vue Flow theme to document
export const applyVueFlowTheme = () => {
  const root = document.documentElement
  
  Object.entries(vueFlowCSSVariables).forEach(([property, value]) => {
    root.style.setProperty(property, value)
  })
}

// Vue Flow node types configuration
export const nodeTypeConfig = {
  // Default node dimensions
  defaultNodeSize: {
    width: 240,
    height: 100
  },
  
  // Node type specific configurations
  nodeTypes: {
    'ai-model': {
      width: 280,
      height: 120,
      color: '#3742fa',
      icon: '🤖'
    },
    'data-connector': {
      width: 260,
      height: 110,
      color: '#2ed573',
      icon: '🔗'
    },
    'logic': {
      width: 220,
      height: 90,
      color: '#ffa502',
      icon: '⚡'
    },
    'trigger': {
      width: 200,
      height: 80,
      color: '#ff4757',
      icon: '🎯'
    }
  }
}

// Connection validation rules
export const connectionRules = {
  // Prevent self-connections
  allowSelfConnections: false,
  
  // Maximum connections per handle
  maxConnectionsPerHandle: 1,
  
  // Connection type validation
  validateConnection: (connection: any) => {
    // Prevent connecting output to output or input to input
    if (connection.sourceHandle?.includes('output') && connection.targetHandle?.includes('output')) {
      return false
    }
    if (connection.sourceHandle?.includes('input') && connection.targetHandle?.includes('input')) {
      return false
    }
    
    return true
  }
}