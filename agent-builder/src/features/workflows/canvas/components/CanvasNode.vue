<script setup lang="ts">
// CanvasNode - Node component based on n8n's implementation
import { computed, ref } from 'vue'
import type { CanvasNodeData } from '@/types/vue-flow'

// Props
interface Props {
  id: string
  data: CanvasNodeData
  selected?: boolean
  dragging?: boolean
  readOnly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  selected: false,
  dragging: false,
  readOnly: false
})

// Emits
const emit = defineEmits<{
  delete: [id: string]
  run: [id: string]
  select: [id: string]
  toggle: [id: string]
  update: [id: string, parameters: Record<string, unknown>]
  move: [id: string, position: { x: number; y: number }]
}>()

// Computed properties
const classes = computed(() => ({
  'canvas-node': true,
  'selected': props.selected,
  'dragging': props.dragging,
  'disabled': props.data.disabled,
  'running': props.data.status === 'running',
  'success': props.data.status === 'success',
  'error': props.data.status === 'error',
  'warning': props.data.status === 'warning'
}))

const nodeType = computed(() => props.data.type)
const nodeName = computed(() => props.data.name)

// Event handlers
const onDelete = () => {
  emit('delete', props.id)
}

const onRun = () => {
  emit('run', props.id)
}

const onSelect = () => {
  emit('select', props.id)
}

const onToggle = () => {
  emit('toggle', props.id)
}

const onUpdate = (parameters: Record<string, unknown>) => {
  emit('update', props.id, parameters)
}

const onMove = (position: { x: number; y: number }) => {
  emit('move', props.id, position)
}

// Icon mapping based on node type
const getNodeIcon = (type: string) => {
  const iconMap: Record<string, string> = {
    'ai-model': '🤖',
    'data-connector': '🔗',
    'logic': '⚡',
    'trigger': '🎯',
    'action': '⚙️',
    'utility': '🔧'
  }
  return iconMap[type] || '📦'
}
</script>

<template>
  <div 
    :class="classes"
    @click="onSelect"
    @dblclick="onRun"
  >
    <!-- Node toolbar (shown on hover) -->
    <div class="node-toolbar" v-if="!readOnly">
      <button class="toolbar-button delete" @click.stop="onDelete" title="Delete node">
        🗑️
      </button>
      <button class="toolbar-button toggle" @click.stop="onToggle" :title="data.disabled ? 'Enable node' : 'Disable node'">
        {{ data.disabled ? '🔓' : '🔒' }}
      </button>
      <button class="toolbar-button run" @click.stop="onRun" title="Run node">
        ▶️
      </button>
    </div>
    
    <!-- Node content -->
    <div class="node-content">
      <div class="node-icon">
        {{ getNodeIcon(nodeType) }}
      </div>
      <div class="node-label">
        {{ nodeName }}
      </div>
      <div v-if="data.disabled" class="node-disabled-label">
        (Disabled)
      </div>
    </div>
    
    <!-- Input handles -->
    <div 
      v-for="(input, index) in data.inputs" 
      :key="`input-${index}`"
      class="node-handle input-handle"
      :data-handle-id="`input-${index}`"
      :data-handle-type="input.type"
    ></div>
    
    <!-- Output handles -->
    <div 
      v-for="(output, index) in data.outputs" 
      :key="`output-${index}`"
      class="node-handle output-handle"
      :data-handle-id="`output-${index}`"
      :data-handle-type="output.type"
    ></div>
  </div>
</template>

<style scoped>
.canvas-node {
  position: relative;
  width: 200px;
  min-height: 80px;
  background: white;
  border: 2px solid #e1e1e1;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.canvas-node:hover {
  border-color: #b1b1b7;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.canvas-node.selected {
  border-color: #ff6d5a;
  box-shadow: 0 0 0 2px rgba(255, 109, 90, 0.2);
}

.canvas-node.disabled {
  opacity: 0.6;
  border-color: #999;
}

.canvas-node.running {
  border-color: #3742fa;
  background: #f0f4ff;
}

.canvas-node.success {
  border-color: #2ed573;
  background: #f0fff5;
}

.canvas-node.error {
  border-color: #ff4757;
  background: #fff0f0;
}

.canvas-node.warning {
  border-color: #ffa502;
  background: #fffaf0;
}

/* Node toolbar */
.node-toolbar {
  position: absolute;
  top: -24px;
  right: 0;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.canvas-node:hover .node-toolbar {
  opacity: 1;
}

.toolbar-button {
  width: 20px;
  height: 20px;
  border: none;
  background: #f0f0f0;
  border-radius: 3px;
  cursor: pointer;
  font-size: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.toolbar-button:hover {
  background: #e0e0e0;
}

/* Node content */
.node-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 100%;
}

.node-icon {
  font-size: 24px;
}

.node-label {
  font-weight: 500;
  font-size: 14px;
  text-align: center;
  word-break: break-word;
}

.node-disabled-label {
  font-size: 12px;
  color: #666;
  font-style: italic;
}

/* Node handles */
.node-handle {
  position: absolute;
  width: 12px;
  height: 12px;
  background: #555;
  border: 2px solid white;
  border-radius: 50%;
  cursor: crosshair;
}

.input-handle {
  left: -6px;
  top: 50%;
  transform: translateY(-50%);
}

.output-handle {
  right: -6px;
  top: 50%;
  transform: translateY(-50%);
}

.node-handle.connectable {
  background: #ff6d5a;
}
</style>