// Application constants following n8n's patterns

export const NODE_CATEGORIES = {
  AI_MODELS: 'ai-models',
  DATA_CONNECTORS: 'data-connectors',
  LOGIC: 'logic',
  TRIGGERS: 'triggers',
  ACTIONS: 'actions',
  UTILITIES: 'utilities'
} as const

export const NODE_EXECUTION_STATUS = {
  IDLE: 'idle',
  RUNNING: 'running',
  SUCCESS: 'success',
  ERROR: 'error',
  WARNING: 'warning'
} as const

export const EXECUTION_STATUS = {
  NEW: 'new',
  RUNNING: 'running',
  SUCCESS: 'success',
  ERROR: 'error',
  CANCELED: 'canceled'
} as const

export const EXECUTION_MODE = {
  MANUAL: 'manual',
  TRIGGER: 'trigger',
  WEBHOOK: 'webhook'
} as const

// Canvas constants
export const CANVAS_DEFAULTS = {
  ZOOM_MIN: 0.1,
  ZOOM_MAX: 3,
  ZOOM_STEP: 0.1,
  NODE_WIDTH: 200,
  NODE_HEIGHT: 100,
  GRID_SIZE: 20
} as const

// API endpoints
export const API_ENDPOINTS = {
  WORKFLOWS: '/v1/agent-builder/workflows',
  NODE_TYPES: '/v1/agent-builder/node-types',
  EXECUTIONS: '/v1/agent-builder/executions',
  WEBSOCKET: '/ws/agent-builder/executions'
} as const

// Local storage keys
export const STORAGE_KEYS = {
  THEME: 'agent-builder-theme',
  WORKSPACE: 'agent-builder-workspace',
  RECENT_WORKFLOWS: 'agent-builder-recent-workflows'
} as const

// Theme colors (matching n8n's color scheme)
export const THEME_COLORS = {
  PRIMARY: '#409EFF',
  SUCCESS: '#67C23A',
  WARNING: '#E6A23C',
  DANGER: '#F56C6C',
  INFO: '#909399',
  
  // Node category colors
  AI_MODELS: '#10B981',
  DATA_CONNECTORS: '#3B82F6',
  LOGIC: '#F59E0B',
  TRIGGERS: '#8B5CF6',
  ACTIONS: '#EF4444',
  UTILITIES: '#6B7280'
} as const