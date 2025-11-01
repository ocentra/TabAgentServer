<template>
  <div 
    :class="[
      'canvas-node',
      nodeTypeClass,
      {
        'selected': selected,
        'disabled': disabled,
        'success': status === 'success',
        'error': status === 'error',
        'running': status === 'running',
        'warning': status === 'warning',
        'trigger': isTrigger
      }
    ]"
    @mouseenter="hovering = true"
    @mouseleave="hovering = false"
  >
    <!-- Node Icon -->
    <div class="node-icon-wrapper">
      <div 
        class="node-icon" 
        :style="{ backgroundColor: nodeColor }"
      >
        <span class="icon-content">{{ nodeIcon }}</span>
      </div>
    </div>

    <!-- Node Description -->
    <div class="node-description">
      <div class="node-label">{{ data.label }}</div>
      <div v-if="data.subtitle" class="node-subtitle">{{ data.subtitle }}</div>
    </div>

    <!-- Status Icons -->
    <div v-if="status && !disabled" class="status-icons">
      <div :class="['status-indicator', status]"></div>
    </div>

    <!-- Warning Indicator (red triangle) -->
    <div v-if="hasWarning" class="warning-indicator">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
      </svg>
    </div>

    <!-- Settings Icon (for configurable nodes) -->
    <div v-if="isConfigurable && !disabled" class="settings-icon">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"></circle>
        <path d="M12 1v6m0 6v6m5.196-2.804l-4.242-4.242M6.804 17.196l4.242-4.242"></path>
      </svg>
    </div>

    <!-- Hover Toolbar -->
    <div v-if="hovering && !readOnly" class="node-toolbar">
      <button @click.stop="handleDelete" title="Delete" class="toolbar-btn">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
      </button>
      <button @click.stop="handleDuplicate" title="Duplicate" class="toolbar-btn">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
      </button>
      <button @click.stop="handleExecute" title="Execute" class="toolbar-btn">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="5 3 19 12 5 21 5 3"></polygon>
        </svg>
      </button>
    </div>

    <!-- Connection Handles -->
    <Handle 
      type="target" 
      :position="Position.Left" 
      class="handle-left"
      :style="{ top: '50%' }"
    />
    <Handle 
      type="source" 
      :position="Position.Right" 
      class="handle-right"
      :style="{ top: '50%' }"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject } from 'vue'
import { Handle, Position } from '@vue-flow/core'

interface Props {
  id: string
  data: {
    label: string
    type?: string
    subtitle?: string
    status?: 'idle' | 'running' | 'success' | 'error' | 'warning'
    disabled?: boolean
    isTrigger?: boolean
    isConfigurable?: boolean
    hasWarning?: boolean
  }
  selected?: boolean
  dragging?: boolean
}

const props = defineProps<Props>()
const hovering = ref(false)

// Inject handlers from parent
const onNodeDelete = inject('onNodeDelete', (id: string) => console.log('Delete:', id))
const onNodeDuplicate = inject('onNodeDuplicate', (id: string) => console.log('Duplicate:', id))
const onNodeExecute = inject('onNodeExecute', (id: string) => console.log('Execute:', id))

// Node action handlers
const handleDelete = () => {
  onNodeDelete(props.id)
}

const handleDuplicate = () => {
  onNodeDuplicate(props.id)
}

const handleExecute = () => {
  onNodeExecute(props.id)
}

// Computed properties
const nodeTypeClass = computed(() => `node-type-${props.data.type || 'default'}`)
const status = computed(() => props.data.status)
const disabled = computed(() => props.data.disabled)
const isTrigger = computed(() => props.data.isTrigger)
const isConfigurable = computed(() => props.data.isConfigurable)
const hasWarning = computed(() => props.data.hasWarning)
const readOnly = computed(() => false)

// Node colors (stolen from n8n)
const nodeColor = computed(() => {
  const colors: Record<string, string> = {
    'ai': '#10B981',        // Green
    'communication': '#3B82F6',  // Blue
    'logic': '#F59E0B',     // Orange
    'data': '#6B7280',      // Gray
    'transform': '#8B5CF6', // Purple
    'trigger': '#FF6D5A'    // n8n red-orange
  }
  return colors[props.data.type || 'default'] || '#6B7280'
})

// Node icons
const nodeIcon = computed(() => {
  const icons: Record<string, string> = {
    'ai': '🤖',
    'communication': '📧',
    'logic': '⚡',
    'data': '💾',
    'transform': '🔄',
    'trigger': '⚡'
  }
  return icons[props.data.type || 'default'] || '📦'
})
</script>

<style scoped>
/* Main node styling (stolen from n8n) */
.canvas-node {
  --canvas-node--border-width: 2px;
  --trigger-node--radius: 36px;
  --node--icon--size: 40px;
  
  position: relative;
  height: 100px;
  width: 240px;
  display: flex;
  align-items: center;
  background: var(--node--color--background, white);
  border: var(--canvas-node--border-width) solid var(--canvas-node--border-color, #ddd);
  border-radius: 8px;
  transition: all 0.2s ease;
  box-shadow: var(--shadow, 0 2px 8px rgba(0, 0, 0, 0.08));
}

/* D-shaped trigger nodes (STOLEN!) */
.canvas-node.trigger {
  border-radius: var(--trigger-node--radius) 8px 8px var(--trigger-node--radius);
}

/* Hover effect */
.canvas-node:hover {
  box-shadow: var(--shadow--dark, 0 4px 12px rgba(0, 0, 0, 0.12));
}

/* Selected state (STOLEN!) */
.canvas-node.selected {
  border-color: #ff6d5a;
  box-shadow: 0 0 0 8px var(--canvas--color--selected, rgba(255, 109, 90, 0.2)),
              0 4px 12px rgba(0, 0, 0, 0.15);
}

/* Status states (STOLEN from n8n!) */
.canvas-node.success {
  --canvas-node--border-color: var(--color--success, #10B981);
}

.canvas-node.error {
  --canvas-node--border-color: var(--color--danger, #EF4444);
}

.canvas-node.running {
  --canvas-node--border-color: var(--node--border-color--running, #3B82F6);
  background-color: var(--node--color--background--executing, #1e293b);
  animation: pulse 2s ease-in-out infinite;
}

.canvas-node.warning {
  --canvas-node--border-color: var(--color--warning, #F59E0B);
}

.canvas-node.disabled {
  --canvas-node--border-color: var(--color--foreground, #9ca3af);
  opacity: 0.6;
}

/* Icon wrapper */
.node-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 12px;
}

.node-icon {
  width: var(--node--icon--size);
  height: var(--node--icon--size);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 20px;
  flex-shrink: 0;
}

.icon-content {
  font-size: 22px;
  line-height: 1;
}

/* Description section */
.node-description {
  flex: 1;
  padding-right: 12px;
  overflow: hidden;
}

.node-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--color--text--shade-1, #333);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-subtitle {
  font-size: 12px;
  color: var(--color--text--tint-1, #777);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

/* Status icons */
.status-icons {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
}

.status-indicator {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.status-indicator.success { background-color: var(--color--success, #10B981); }
.status-indicator.error { background-color: var(--color--danger, #EF4444); }
.status-indicator.running { 
  background-color: var(--color--info, #3B82F6);
  animation: pulse 2s ease-in-out infinite;
}
.status-indicator.warning { background-color: var(--color--warning, #F59E0B); }
.status-indicator.idle { background-color: var(--node--border-color, #9ca3af); }

/* Warning indicator (red triangle - STOLEN!) */
.warning-indicator {
  position: absolute;
  top: -6px;
  right: -6px;
  color: var(--color--danger, #EF4444);
  background: var(--node--color--background, white);
  border-radius: 50%;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1;
}

/* Settings icon */
.settings-icon {
  position: absolute;
  top: 8px;
  right: 8px;
  color: var(--color--text--tint-1, #777);
  opacity: 0.6;
}

/* Node toolbar (improved) */
.node-toolbar {
  position: absolute;
  top: -40px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--color--background--light-2, white);
  border: 1px solid var(--color--foreground--shade-2, #ddd);
  border-radius: 6px;
  padding: 4px;
  display: flex;
  gap: 4px;
  box-shadow: var(--shadow, 0 2px 8px rgba(0, 0, 0, 0.1));
  z-index: 10;
}

.toolbar-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--color--text, #333);
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.toolbar-btn:hover {
  background: var(--color--background--light-1, #f5f5f5);
}

/* Connection handles (improved) */
:deep(.vue-flow__handle) {
  width: 12px;
  height: 12px;
  background: var(--color--foreground, #666);
  border: 2px solid var(--node--color--background, white);
  border-radius: 50%;
  transition: all 0.2s;
}

:deep(.vue-flow__handle:hover) {
  background: #ff6d5a;
  transform: scale(1.2);
}

:deep(.vue-flow__handle.connecting) {
  background: #ff6d5a;
}

/* Pulse animation for running state */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}
</style>

