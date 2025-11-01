<template>
  <div 
    :class="['custom-node', { selected, dragging, disabled }]"
    @mouseenter="hovering = true"
    @mouseleave="hovering = false"
  >
    <!-- Node Icon -->
    <div class="node-icon" :style="{ background: nodeColor }">
      <span class="icon-text">{{ nodeIcon }}</span>
    </div>
    
    <!-- Node Label -->
    <div class="node-label">{{ data.label }}</div>
    
    <!-- Status Indicator -->
    <div v-if="status" :class="['status-indicator', status]"></div>
    
    <!-- Hover Toolbar -->
    <div v-if="hovering && !readOnly" class="node-toolbar">
      <button @click.stop="handleDelete" title="Delete">🗑️</button>
      <button @click.stop="handleDuplicate" title="Duplicate">📋</button>
      <button @click.stop="handleExecute" title="Execute">▶️</button>
    </div>
    
    <!-- Connection Handles -->
    <Handle 
      type="target" 
      position="left" 
      class="handle-left"
      :style="{ top: '50%' }"
    />
    <Handle 
      type="source" 
      position="right" 
      class="handle-right"
      :style="{ top: '50%' }"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject } from 'vue'
import { Handle } from '@vue-flow/core'

interface Props {
  id: string
  data: {
    label: string
    type?: string
    status?: 'idle' | 'running' | 'success' | 'error'
    disabled?: boolean
  }
  selected?: boolean
  dragging?: boolean
}

const props = defineProps<Props>()
const hovering = ref(false)

// Inject handlers from parent (SimpleCanvas)
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

// Node color based on type
const nodeColor = computed(() => {
  const colors: Record<string, string> = {
    'ai': '#10B981',
    'data': '#3B82F6',
    'logic': '#F59E0B',
    'communication': '#EF4444',
    'transform': '#06B6D4',
    'trigger': '#8B5CF6'
  }
  return colors[props.data.type || 'default'] || '#6B7280'
})

// Node icon emoji based on type
const nodeIcon = computed(() => {
  const icons: Record<string, string> = {
    'ai': '🤖',
    'data': '💾',
    'logic': '⚡',
    'communication': '📧',
    'transform': '🔄',
    'trigger': '🎯'
  }
  return icons[props.data.type || 'default'] || '📦'
})

const status = computed(() => props.data.status)
const readOnly = computed(() => false)
const disabled = computed(() => props.data.disabled)
</script>

<style scoped>
.custom-node {
  position: relative;
  background: var(--node--color--background, white);
  border: 2px solid var(--node--border-color, #ddd);
  border-radius: 12px;
  padding: 16px;
  min-width: 200px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  transition: all 0.2s ease;
  cursor: grab;
}

.custom-node:hover {
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  transform: translateY(-2px);
}

.custom-node.selected {
  border-color: #ff6d5a !important;
  box-shadow: 0 0 0 3px rgba(255, 109, 90, 0.3), 0 4px 12px rgba(0, 0, 0, 0.15);
}

.custom-node.dragging {
  cursor: grabbing;
  opacity: 0.8;
}

.custom-node.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.node-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 12px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.icon-text {
  font-size: 24px;
  line-height: 1;
  filter: brightness(1.2);
}

.node-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--color--text--shade-1, #333);
  text-align: center;
}

.status-indicator {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid var(--node--color--background, white);
}

.status-indicator.idle {
  background: #9CA3AF;
}

.status-indicator.running {
  background: #3B82F6;
  animation: pulse 1.5s infinite;
}

.status-indicator.success {
  background: #10B981;
}

.status-indicator.error {
  background: #EF4444;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.node-toolbar {
  position: absolute;
  top: -40px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--color--background--light-2, white);
  border: 1px solid var(--color--foreground--shade-2, #ddd);
  border-radius: 8px;
  padding: 4px;
  display: flex;
  gap: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  opacity: 0;
  animation: fadeIn 0.2s ease forwards;
}

@keyframes fadeIn {
  to { opacity: 1; }
}

.node-toolbar button {
  background: transparent;
  border: none;
  padding: 6px 10px;
  cursor: pointer;
  border-radius: 6px;
  font-size: 16px;
  transition: background 0.2s;
}

.node-toolbar button:hover {
  background: var(--color--background--light-1, #f5f5f5);
}

/* Connection Handles */
:deep(.handle-left),
:deep(.handle-right) {
  width: 14px;
  height: 14px;
  background: var(--color--foreground, #666);
  border: 3px solid var(--node--color--background, white);
  border-radius: 50%;
  transition: all 0.2s;
}

:deep(.handle-left):hover,
:deep(.handle-right):hover {
  background: #ff6d5a;
  transform: scale(1.3);
  box-shadow: 0 0 0 4px rgba(255, 109, 90, 0.2);
}

:deep(.handle-left) {
  left: -7px;
}

:deep(.handle-right) {
  right: -7px;
}
</style>

