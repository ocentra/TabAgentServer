<script setup lang="ts">
// WorkflowCanvas - Main workflow editor component with Vue Flow integration
import { ref, computed, onMounted } from 'vue'
import { VueFlow, useVueFlow, type Node, type Edge } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
// import { MiniMap } from '@vue-flow/minimap' // TODO: Fix version compatibility
import type { CanvasNode, CanvasConnection } from '@/types/vue-flow'
import { defaultVueFlowConfig } from '@/utils/vue-flow-config'

// Props
interface Props {
  workflowId?: string
  readOnly?: boolean
  executing?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  readOnly: false,
  executing: false
})

// Emits
const emit = defineEmits<{
  'workflow:save': [workflow: any]
  'workflow:execute': [workflowId: string]
  'node:select': [nodeId: string]
}>()

// Vue Flow instance
const { 
  nodes, 
  edges, 
  addNodes, 
  addEdges, 
  onConnect,
  onNodeClick,
  onPaneClick,
  onPaneContextMenu,
  onNodeDragStop,
  onSelectionDragStop,
  onSelectionEnd,
  project,
  viewport,
  dimensions,
  fitView,
  zoomIn,
  zoomOut,
  zoomTo,
  setViewport
} = useVueFlow()

// Demo nodes and edges
const initialNodes = ref<CanvasNode[]>([
  {
    id: '1',
    type: 'default',
    position: { x: 100, y: 100 },
    data: {
      type: 'ai-model',
      name: 'GPT-4 Node',
      parameters: {},
      position: { x: 100, y: 100 },
      inputs: [{ name: 'prompt', type: 'string', required: true }],
      outputs: [{ name: 'response', type: 'string' }],
      status: 'idle'
    },
    label: 'GPT-4 Model'
  },
  {
    id: '2',
    type: 'default',
    position: { x: 400, y: 100 },
    data: {
      type: 'data-connector',
      name: 'Email Connector',
      parameters: {},
      position: { x: 400, y: 100 },
      inputs: [{ name: 'content', type: 'string', required: true }],
      outputs: [{ name: 'sent', type: 'boolean' }],
      status: 'idle'
    },
    label: 'Email Connector'
  },
  {
    id: '3',
    type: 'default',
    position: { x: 250, y: 250 },
    data: {
      type: 'logic',
      name: 'Condition',
      parameters: {},
      position: { x: 250, y: 250 },
      inputs: [{ name: 'input', type: 'any', required: true }],
      outputs: [{ name: 'true', type: 'any' }, { name: 'false', type: 'any' }],
      status: 'idle'
    },
    label: 'Condition'
  }
])

const initialEdges = ref<CanvasConnection[]>([
  {
    id: 'e1-2',
    source: '1',
    target: '2',
    sourceHandle: 'output-1',
    targetHandle: 'input-1',
    type: 'smoothstep',
    animated: false
  },
  {
    id: 'e1-3',
    source: '1',
    target: '3',
    sourceHandle: 'output-1',
    targetHandle: 'input-1',
    type: 'smoothstep',
    animated: false
  }
])

// Event handlers
const handleConnect = (connection: any) => {
  console.log('Connection created:', connection)
  // Add the new connection to the flow
  addEdges([{
    ...connection,
    id: `e${connection.source}-${connection.target}`,
    type: 'smoothstep',
    animated: false
  }])
}

const handleNodeClick = (event: any) => {
  console.log('Node clicked:', event.node)
  emit('node:select', event.node.id)
}

const handlePaneClick = (event: any) => {
  console.log('Pane clicked:', event)
}

const handlePaneContextMenu = (event: any) => {
  console.log('Pane context menu:', event)
}

const handleNodeDragStop = (event: any) => {
  console.log('Node drag stop:', event)
}

const handleSelectionDragStop = (event: any) => {
  console.log('Selection drag stop:', event)
}

const handleSelectionEnd = (event: any) => {
  console.log('Selection end:', event)
}

// Initialize demo
onMounted(() => {
  // Add initial nodes and edges
  addNodes(initialNodes.value)
  addEdges(initialEdges.value)
  
  // Set up event listeners
  onConnect(handleConnect)
  onNodeClick(handleNodeClick)
  onPaneClick(handlePaneClick)
  onPaneContextMenu(handlePaneContextMenu)
  onNodeDragStop(handleNodeDragStop)
  onSelectionDragStop(handleSelectionDragStop)
  onSelectionEnd(handleSelectionEnd)
  
  // Fit view to show all nodes
  setTimeout(() => {
    fitView({ padding: 0.5 })
  }, 100)
})

// View controls
const onZoomIn = () => {
  zoomIn()
}

const onZoomOut = () => {
  zoomOut()
}

const onFitView = () => {
  fitView({ padding: 0.5 })
}

const onResetZoom = () => {
  zoomTo(1)
}
</script>

<template>
  <div class="workflow-canvas">
    <VueFlow
      v-bind="defaultVueFlowConfig"
      :class="{ 'read-only': readOnly, 'executing': executing }"
      class="vue-flow-container"
    >
      <!-- Background -->
      <Background 
        pattern-color="#e1e1e1"
        :gap="16"
      />
      
      <!-- Controls -->
      <Controls 
        position="bottom-left"
        :show-zoom="true"
        :show-fit-view="true"
        :show-interactive="true"
        @zoom-in="onZoomIn"
        @zoom-out="onZoomOut"
        @fit-view="onFitView"
      />
      
      <!-- MiniMap - TODO: Fix version compatibility -->
      <!-- <MiniMap 
        position="bottom-right"
        :node-color="(node) => node.data?.status === 'running' ? '#3742fa' : '#ffffff'"
        :node-stroke-color="() => '#e1e1e1'"
        :node-stroke-width="2"
        :pannable="true"
        :zoomable="true"
      /> -->
    </VueFlow>
  </div>
</template>

<style scoped>
.workflow-canvas {
  height: 100%;
  width: 100%;
  position: relative;
}

.vue-flow-container {
  width: 100%;
  height: 100%;
}

.vue-flow-container.read-only {
  cursor: not-allowed;
}

.vue-flow-container.executing {
  /* Add visual indication for executing state */
}

/* Vue Flow custom styles */
:deep(.vue-flow__node) {
  background: white;
  border: 2px solid #e1e1e1;
  border-radius: 8px;
  padding: 10px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  font-size: 12px;
  color: #333;
  min-width: 150px;
  text-align: center;
}

:deep(.vue-flow__node.selected) {
  border-color: #ff6d5a;
  box-shadow: 0 0 0 2px rgba(255, 109, 90, 0.2);
}

:deep(.vue-flow__edge-path) {
  stroke: #b1b1b7;
  stroke-width: 2px;
}

:deep(.vue-flow__edge.selected .vue-flow__edge-path) {
  stroke: #ff6d5a;
  stroke-width: 3px;
}

:deep(.vue-flow__handle) {
  width: 8px;
  height: 8px;
  background: #555;
  border: 2px solid white;
}

:deep(.vue-flow__handle.connectable) {
  cursor: crosshair;
}

:deep(.vue-flow__handle.connecting) {
  background: #ff6d5a;
}
</style>