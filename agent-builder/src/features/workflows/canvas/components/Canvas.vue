<template>
  <div class="canvas-wrapper">
    <VueFlow
      v-model="elements"
      :node-types="nodeTypes"
      :edge-types="edgeTypes"
      :default-zoom="1"
      :min-zoom="0.1"
      :max-zoom="4"
      :auto-pan-on-node-drag="true"
      :select-nodes-on-drag="false"
      class="workflow-canvas"
      @drop="onDrop"
      @dragover="onDragOver"
      @node-drag="onNodeDrag"
      @node-drag-stop="onNodeDragStop"
      @node-click="onNodeClick"
      @pane-click="onPaneClick"
      @connect="onConnect"
      @connect-start="onConnectStart"
      @connect-end="onConnectEnd"
    >
      <Background :pattern-color="isDark ? '#444' : '#ccc'" :gap="20" />
      <Controls />
    </VueFlow>
  </div>
</template>

<script setup lang="ts">
import { ref, provide, onMounted } from 'vue'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import EnhancedNode from '@/components/canvas/EnhancedNode.vue'
import SmartEdge from '@/components/canvas/SmartEdge.vue'

// Define emits
const emit = defineEmits(['node-click', 'pane-click'])

// Register enhanced node and smart edge types
const nodeTypes = {
  custom: EnhancedNode
}

const edgeTypes = {
  smart: SmartEdge
}

// Set dark theme on mount
onMounted(() => {
  document.documentElement.setAttribute('data-theme', 'dark')
})

// Vue Flow instance (combined destructuring)
const { project, addNodes, removeNodes, getNode } = useVueFlow()

// Drag & Drop constants (matching n8n)
const DRAG_EVENT_DATA_KEY = 'nodeData'

// Node ID counter
const nodeIdCounter = ref(6)

// Drag & Drop handlers (copied from n8n)
const onDragOver = (event: DragEvent) => {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy'
  }
}

const onDrop = (event: DragEvent) => {
  event.preventDefault()
  
  const dragData = event.dataTransfer?.getData(DRAG_EVENT_DATA_KEY)
  if (!dragData) return
  
  try {
    const nodeData = JSON.parse(dragData)
    
    // Get the drop position on the canvas
    const position = project({ x: event.clientX, y: event.clientY })
    
    // Create new node with unique ID
    const newNode = {
      id: `node-${nodeIdCounter.value++}`,
      type: 'custom',
      position: { x: position.x - 100, y: position.y - 40 }, // Center on cursor
      data: {
        label: nodeData.displayName,
        type: nodeData.type,
        status: 'idle',
        color: nodeData.color,
        icon: nodeData.icon
      }
    }
    
    // Add node to canvas
    addNodes([newNode])
  } catch (error) {
    console.error('Failed to parse drag data:', error)
  }
}

// Handle node dragging for smooth movement (STOLEN from n8n!)
const onNodeDrag = (event: any) => {
  // This fires during drag to update edges in real-time
  // Makes dragging feel smooth and responsive!
}

// Handle node drag stop (node movement on canvas)
const onNodeDragStop = (event: any) => {
  console.log('Node moved:', event)
  // Node position is automatically updated by Vue Flow
}

// Handle node actions (delete, duplicate, execute)
const handleNodeDelete = (nodeId: string) => {
  removeNodes([nodeId])
  console.log('Deleted node:', nodeId)
}

const handleNodeDuplicate = (nodeId: string) => {
  const node = getNode(nodeId)
  if (!node) return
  
  const newNode = {
    ...node,
    id: `node-${nodeIdCounter.value++}`,
    position: {
      x: node.position.x + 50,
      y: node.position.y + 50
    }
  }
  
  addNodes([newNode])
  console.log('Duplicated node:', nodeId)
}

const handleNodeExecute = (nodeId: string) => {
  console.log('Execute node:', nodeId)
  // TODO: Implement execution logic
}

// Provide handlers to child components (CustomNode)
provide('onNodeDelete', handleNodeDelete)
provide('onNodeDuplicate', handleNodeDuplicate)
provide('onNodeExecute', handleNodeExecute)

// Node click handler
const onNodeClick = (event: any) => {
  console.log('Node clicked:', event.node)
  emit('node-click', event.node)
}

// Pane (empty space) click handler
const onPaneClick = () => {
  console.log('Canvas pane clicked - hiding Properties Panel')
  emit('pane-click')
}

// Connection handlers (copying n8n pattern)
const onConnectStart = (event: any) => {
  console.log('Connection started:', event)
}

const onConnect = (connection: any) => {
  console.log('Connection created:', connection)
  // Connection is automatically added by Vue Flow
}

const onConnectEnd = (event: any) => {
  console.log('Connection ended:', event)
}

// Demo workflow nodes with enhanced styling (stolen features from n8n!)
const elements = ref([
  // Trigger node with D-shape
  {
    id: '1',
    type: 'custom',
    position: { x: 50, y: 100 },
    data: {
      label: 'Gmail Trigger',
      subtitle: 'On new email',
      type: 'trigger',
      status: 'success',
      isTrigger: true,
      hasWarning: true
    },
  },
  // Regular node
  {
    id: '2',
    type: 'custom',
    position: { x: 350, y: 100 },
    data: {
      label: 'Extract Data',
      subtitle: 'Parse email content',
      type: 'transform',
      status: 'idle'
    },
  },
  // Node with warning
  {
    id: '3',
    type: 'custom',
    position: { x: 650, y: 100 },
    data: {
      label: 'OpenAI GPT-4',
      subtitle: 'Analyze sentiment',
      type: 'ai',
      status: 'running',
      hasWarning: true
    },
  },
  // Error node
  {
    id: '4',
    type: 'custom',
    position: { x: 350, y: 280 },
    data: {
      label: 'Save to DB',
      subtitle: 'PostgreSQL',
      type: 'data',
      status: 'error'
    },
  },
  // Success node
  {
    id: '5',
    type: 'custom',
    position: { x: 650, y: 280 },
    data: {
      label: 'Send Slack',
      subtitle: '#notifications',
      type: 'communication',
      status: 'success'
    },
  },
  {
    id: 'e1-2',
    source: '1',
    target: '2',
    type: 'smart', // Smart routing that avoids nodes!
    animated: false,
    style: { stroke: '#10B981', strokeWidth: 2 }
  },
  {
    id: 'e1-3',
    source: '1',
    target: '3',
    type: 'smart', // Intelligent path calculation
    animated: true,
    style: { stroke: '#3B82F6', strokeWidth: 2 }
  },
  {
    id: 'e3-4',
    source: '3',
    target: '4',
    type: 'smart', // Routes around nodes like n8n!
    style: { stroke: '#999', strokeWidth: 2 }
  },
  {
    id: 'e3-5',
    source: '3',
    target: '5',
    type: 'smart', // Smart routing FTW!
    style: { stroke: '#EF4444', strokeWidth: 2, strokeDasharray: '5,5' }
  }
])
</script>

<style scoped>
.canvas-wrapper {
  width: 100%;
  height: 100vh;
  background: var(--color--background, #f6f6f6);
  position: relative;
}

.workflow-canvas {
  width: 100%;
  height: 100%;
}

/* Remove default node styling - custom nodes handle their own */
:deep(.vue-flow__node) {
  padding: 0 !important;
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
}

/* Remove any grey container div */
:deep(.vue-flow__node-default),
:deep(.vue-flow__node-input),
:deep(.vue-flow__node-output) {
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
}

/* Smooth Bezier curves like n8n! */
:deep(.vue-flow__edge-path) {
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: stroke 0.3s ease, stroke-width 0.3s ease;
}

:deep(.vue-flow__edge.selected .vue-flow__edge-path) {
  stroke: #ff6d5a !important;
  stroke-width: 3px;
}

:deep(.vue-flow__edge:hover .vue-flow__edge-path) {
  stroke-width: 3px;
}
</style>

