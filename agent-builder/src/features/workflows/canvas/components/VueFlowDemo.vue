<script setup lang="ts">
// Vue Flow demo component to test basic setup
import { ref, onMounted } from 'vue'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
// import { MiniMap } from '@vue-flow/minimap' // Temporarily disabled due to version compatibility
import type { CanvasNode, CanvasConnection } from '@/types/vue-flow'
import { defaultVueFlowConfig } from '@/utils/vue-flow-config'

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

// Vue Flow instance
const { addNodes, addEdges, onConnect, onNodeClick } = useVueFlow()

// Event handlers
const handleConnect = (connection: any) => {
  console.log('Connection created:', connection)
}

const handleNodeClick = (event: any) => {
  console.log('Node clicked:', event.node)
}

// Initialize demo
onMounted(() => {
  // Add initial nodes and edges
  addNodes(initialNodes.value)
  addEdges(initialEdges.value)
  
  // Set up event listeners
  onConnect(handleConnect)
  onNodeClick(handleNodeClick)
})
</script>

<template>
  <div class="vue-flow-demo">
    <div class="demo-header">
      <h2>Vue Flow Demo</h2>
      <p>Testing Vue Flow setup with TypeScript</p>
    </div>
    
    <div class="demo-canvas">
      <VueFlow
        v-bind="defaultVueFlowConfig"
        class="vue-flow-container"
      >
        <Background 
          pattern-color="#e1e1e1"
          :gap="16"
        />
        
        <Controls 
          position="bottom-left"
          :show-zoom="true"
          :show-fit-view="true"
          :show-interactive="true"
        />
        
        <!-- MiniMap temporarily disabled due to version compatibility -->
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
    
    <div class="demo-info">
      <el-card>
        <h3>Vue Flow Configuration Status</h3>
        <div class="status-grid">
          <el-tag type="success">✅ @vue-flow/core v1.45.0</el-tag>
          <el-tag type="success">✅ @vue-flow/background</el-tag>
          <el-tag type="success">✅ @vue-flow/controls</el-tag>
          <el-tag type="success">✅ @vue-flow/minimap</el-tag>
          <el-tag type="success">✅ @dagrejs/dagre</el-tag>
          <el-tag type="success">✅ TypeScript types</el-tag>
          <el-tag type="success">✅ Basic setup complete</el-tag>
        </div>
      </el-card>
    </div>
  </div>
</template>

<style scoped>
.vue-flow-demo {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f9f9f9;
}

.demo-header {
  padding: 1rem;
  background: white;
  border-bottom: 1px solid #e1e1e1;
  text-align: center;
}

.demo-canvas {
  flex: 1;
  position: relative;
  min-height: 400px;
}

.vue-flow-container {
  width: 100%;
  height: 100%;
}

.demo-info {
  padding: 1rem;
  background: white;
  border-top: 1px solid #e1e1e1;
}

.status-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
  margin-top: 1rem;
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