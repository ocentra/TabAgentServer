<template>
  <div class="workflow-editor">
    <!-- Main Content -->
    <div class="main-content">
      <!-- Top Bar -->
      <TopBar 
        @save="handleWorkflowSave"
        @share="handleShare"
        @duplicate="handleDuplicate"
        @delete="handleDelete"
        @export="handleExport"
      />
      
      <!-- Main Editor Layout -->
      <div class="editor-layout">
      <!-- Sidebar with Node Library -->
      <aside 
        v-if="nodeLibraryOpen" 
        class="node-library-sidebar"
        :class="{ collapsed: nodeLibraryCollapsed }"
        :style="{ width: leftPanelWidth + 'px' }"
      >
        <!-- Collapse Button - TOP when collapsed -->
        <button @click="toggleNodeLibrary" class="collapse-btn" :class="{ top: nodeLibraryCollapsed }">
          {{ nodeLibraryCollapsed ? '→' : '←' }}
        </button>
        
        <!-- Content -->
        <div v-if="!nodeLibraryCollapsed" class="panel-content">
          <NodeLibrary @open-settings="openSettings" />
        </div>
        
        <!-- Resize Handle -->
        <div 
          v-if="!nodeLibraryCollapsed"
          class="resize-handle right"
          @mousedown="startResize('left', $event)"
        ></div>
      </aside>

      <!-- Main Canvas Area -->
      <main class="canvas-container">
            <Canvas @node-click="openNodeProperties" @pane-click="closePropertiesPanel" />
      </main>

      <!-- Properties Panel -->
      <aside 
        v-if="propertiesPanelOpen" 
        class="properties-panel"
        :class="{ collapsed: propertiesPanelCollapsed }"
        :style="{ width: rightPanelWidth + 'px' }"
      >
        <!-- Collapse Button - TOP when collapsed -->
        <button @click="togglePropertiesPanel" class="collapse-btn" :class="{ top: propertiesPanelCollapsed }">
          {{ propertiesPanelCollapsed ? '←' : '→' }}
        </button>
        
        <!-- Resize Handle -->
        <div 
          v-if="!propertiesPanelCollapsed"
          class="resize-handle left"
          @mousedown="startResize('right', $event)"
        ></div>
        
        <!-- Content -->
        <div v-if="!propertiesPanelCollapsed" class="panel-content">
          <PropertiesPanel 
            :type="propertiesPanelType"
            :selected-node="selectedNode"
          />
        </div>
      </aside>
      </div>

      <!-- Execution Panel (bottom) -->
      <div 
        v-if="uiStore.executionPanelOpen" 
        class="execution-panel"
      >
        <ExecutionPanel />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useUIStore } from '@/stores'
// import { useWorkflowStore } from '@/stores' // Will be used in future tasks
import Canvas from '@/features/workflows/canvas/components/Canvas.vue'
import NodeLibrary from '@/features/workflows/library/components/NodeLibrary.vue'
import PropertiesPanel from '@/features/workflows/properties/components/PropertiesPanel.vue'
import ExecutionPanel from '@/features/workflows/execution/components/ExecutionPanel.vue'
import TopBar from '@/components/TopBar.vue'

interface Props {
  id?: string
  readOnly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  readOnly: false
})

const route = useRoute()
const uiStore = useUIStore()
// const workflowStore = useWorkflowStore() // Will be used in future tasks

const workflowId = ref<string | undefined>(props.id)

// Panel state
const nodeLibraryOpen = ref(true)
const nodeLibraryCollapsed = ref(false)
const leftPanelWidth = ref(300)

const propertiesPanelOpen = ref(true)
const propertiesPanelCollapsed = ref(false)
const rightPanelWidth = ref(350)

// Properties Panel content type
const propertiesPanelType = ref<'settings' | 'node' | null>(null)
const selectedNode = ref<any>(null)

// Resizing logic
const resizing = ref<'left' | 'right' | null>(null)
const startX = ref(0)
const startWidth = ref(0)

const startResize = (side: 'left' | 'right', event: MouseEvent) => {
  resizing.value = side
  startX.value = event.clientX
  startWidth.value = side === 'left' ? leftPanelWidth.value : rightPanelWidth.value
  
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
  event.preventDefault()
}

const handleResize = (event: MouseEvent) => {
  if (!resizing.value) return
  
  const delta = event.clientX - startX.value
  const newWidth = resizing.value === 'left' 
    ? startWidth.value + delta 
    : startWidth.value - delta
  
  // Min/max width constraints (min width = 200, max = 600)
  const constrainedWidth = Math.max(200, Math.min(600, newWidth))
  
  if (resizing.value === 'left') {
    leftPanelWidth.value = constrainedWidth
    // If panel gets too narrow, auto-collapse
    if (constrainedWidth < 100) {
      nodeLibraryCollapsed.value = true
      leftPanelWidth.value = 48
    } else {
      nodeLibraryCollapsed.value = false
    }
  } else {
    rightPanelWidth.value = constrainedWidth
    // If panel gets too narrow, auto-collapse
    if (constrainedWidth < 100) {
      propertiesPanelCollapsed.value = true
      rightPanelWidth.value = 48
    } else {
      propertiesPanelCollapsed.value = false
    }
  }
}

const stopResize = () => {
  resizing.value = null
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
}

const toggleNodeLibrary = () => {
  if (nodeLibraryCollapsed.value) {
    // Expanding
    nodeLibraryCollapsed.value = false
    leftPanelWidth.value = 300
  } else {
    // Collapsing
    nodeLibraryCollapsed.value = true
    leftPanelWidth.value = 24  // Super thin when collapsed
  }
}

const togglePropertiesPanel = () => {
  if (propertiesPanelCollapsed.value) {
    // Expanding
    propertiesPanelCollapsed.value = false
    rightPanelWidth.value = 350
  } else {
    // Collapsing
    propertiesPanelCollapsed.value = true
    rightPanelWidth.value = 24  // Super thin when collapsed
  }
}

// Watch for route changes
watch(() => route.params.id, (newId) => {
  if (typeof newId === 'string') {
    workflowId.value = newId
    loadWorkflow(newId)
  }
}, { immediate: true })

onMounted(() => {
  if (workflowId.value) {
    loadWorkflow(workflowId.value)
  } else {
    // Create new workflow
    createNewWorkflow()
  }
})

async function loadWorkflow(id: string) {
  try {
    uiStore.setLoading(true)
    // TODO: Implement workflow loading from API
    console.log('Loading workflow:', id)
  } catch (error) {
    console.error('Failed to load workflow:', error)
    uiStore.addNotification({
      type: 'error',
      message: 'Failed to load workflow'
    })
  } finally {
    uiStore.setLoading(false)
  }
}

function createNewWorkflow() {
  // TODO: Create new workflow
  console.log('Creating new workflow')
}

function handleWorkflowSave(workflow?: any) {
  // TODO: Implement workflow saving
  console.log('Saving workflow:', workflow)
  uiStore.addNotification({
    type: 'success',
    message: 'Workflow saved successfully'
  })
}

function handleShare() {
  console.log('Sharing workflow')
  uiStore.addNotification({
    type: 'info',
    message: 'Share functionality coming soon'
  })
}

function handleDuplicate() {
  console.log('Duplicating workflow')
  uiStore.addNotification({
    type: 'info',
    message: 'Workflow duplicated'
  })
}

function handleDelete() {
  console.log('Deleting workflow')
  // TODO: Add confirmation dialog
  uiStore.addNotification({
    type: 'warning',
    message: 'Delete functionality requires confirmation'
  })
}

function handleExport() {
  console.log('Exporting workflow')
  uiStore.addNotification({
    type: 'info',
    message: 'Export functionality coming soon'
  })
}

function handleWorkflowExecute(workflowId: string) {
  // TODO: Implement workflow execution
  console.log('Executing workflow:', workflowId)
  uiStore.toggleExecutionPanel()
}

// Open Settings in Properties Panel
function openSettings() {
  propertiesPanelType.value = 'settings'
  selectedNode.value = null
  
  // Auto-expand Properties Panel
  if (propertiesPanelCollapsed.value) {
    propertiesPanelCollapsed.value = false
    rightPanelWidth.value = 350
  }
  
  // Ensure panel is open
  propertiesPanelOpen.value = true
}

// Open Node config in Properties Panel
function openNodeProperties(node: any) {
  propertiesPanelType.value = 'node'
  selectedNode.value = node
  
  // Auto-expand Properties Panel
  if (propertiesPanelCollapsed.value) {
    propertiesPanelCollapsed.value = false
    rightPanelWidth.value = 350
  }
  
  // Ensure panel is open
  propertiesPanelOpen.value = true
}

// Close Properties Panel when clicking empty canvas
function closePropertiesPanel() {
  // Collapse the Properties Panel
  if (!propertiesPanelCollapsed.value) {
    propertiesPanelCollapsed.value = true
    rightPanelWidth.value = 24
  }
  
  // Clear selected node
  selectedNode.value = null
  propertiesPanelType.value = null
}
</script>

<style scoped>
.workflow-editor {
  height: 100vh;
  display: flex;
  flex-direction: row;
  background: var(--color-background);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.node-library-sidebar,
.properties-panel {
  position: relative;
  background: var(--color-background-light);
  overflow-y: auto;
  color: var(--color--text--shade-1);
  transition: width 0.3s ease;
}

.node-library-sidebar {
  border-right: 1px solid var(--color-border);
}

.properties-panel {
  border-left: 1px solid var(--color-border);
}

/* Panel content */
.panel-content {
  height: 100%;
  overflow-y: auto;
  padding: 0;
}

/* Collapsed state - minimal width */
.node-library-sidebar.collapsed,
.properties-panel.collapsed {
  overflow: visible !important;
}

.canvas-container {
  flex: 1;
  position: relative;
  overflow: hidden;
  background: var(--canvas--color--background);
}

/* Collapse Buttons */
.collapse-btn {
  position: absolute;
  z-index: 10;
  background: var(--color--background--light-2, white);
  border: 1px solid var(--color--foreground--shade-2, #ddd);
  border-radius: 4px;
  width: 24px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* Default position - center */
.node-library-sidebar .collapse-btn {
  top: 50%;
  transform: translateY(-50%);
  right: -12px;
}

.properties-panel .collapse-btn {
  top: 50%;
  transform: translateY(-50%);
  left: -12px;
}

/* When collapsed - move to top */
.collapse-btn.top {
  top: 12px;
  transform: none;
}

.collapse-btn:hover {
  background: var(--color--background--light-1, #f5f5f5);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

/* Resize Handles */
.resize-handle {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  background: transparent;
  transition: background 0.2s;
}

.resize-handle:hover {
  background: var(--color--primary, #ff6d5a);
}

.resize-handle.right {
  right: 0;
}

.resize-handle.left {
  left: 0;
}

.execution-panel {
  height: 300px;
  background: var(--color-background-light);
  border-top: 1px solid var(--color-border);
  overflow-y: auto;
}

/* CSS Variables for theming */
:root {
  --color-background: #f6f6f6;
  --color-background-light: #ffffff;
  --color-border: #e1e5e9;
  --color-text-primary: #2c3e50;
}

[data-theme="dark"] {
  --color-background: #1e1e1e;
  --color-background-light: #2d2d2d;
  --color-border: #404040;
  --color-text-primary: #e0e0e0;
  --canvas--color--background: #2d2e2e;
  --node--color--background: #3a3a3a;
  --node--border-color: #555;
}
</style>