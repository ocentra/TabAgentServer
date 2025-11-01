import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { WorkflowData, CanvasNode, CanvasConnection } from '@/types'

export const useWorkflowStore = defineStore('workflow', () => {
  // State
  const currentWorkflow = ref<WorkflowData | null>(null)
  const workflows = ref<WorkflowData[]>([])
  const activeNodeId = ref<string | null>(null)
  const selectedNodeIds = ref<string[]>([])
  const isDirty = ref(false)

  // Getters
  const hasActiveWorkflow = computed(() => currentWorkflow.value !== null)
  const activeNode = computed(() => {
    if (!activeNodeId.value || !currentWorkflow.value) return null
    return currentWorkflow.value.nodes.find(node => node.id === activeNodeId.value) || null
  })

  // Actions
  function setCurrentWorkflow(workflow: WorkflowData | null) {
    currentWorkflow.value = workflow
    isDirty.value = false
  }

  function setActiveNode(nodeId: string | null) {
    activeNodeId.value = nodeId
  }

  function setSelectedNodes(nodeIds: string[]) {
    selectedNodeIds.value = nodeIds
  }

  function addNode(node: CanvasNode) {
    if (!currentWorkflow.value) return
    currentWorkflow.value.nodes.push(node)
    isDirty.value = true
  }

  function removeNode(nodeId: string) {
    if (!currentWorkflow.value) return
    const index = currentWorkflow.value.nodes.findIndex(node => node.id === nodeId)
    if (index !== -1) {
      currentWorkflow.value.nodes.splice(index, 1)
      isDirty.value = true
    }
  }

  function updateNode(nodeId: string, updates: Partial<CanvasNode>) {
    if (!currentWorkflow.value) return
    const node = currentWorkflow.value.nodes.find(n => n.id === nodeId)
    if (node) {
      Object.assign(node, updates)
      isDirty.value = true
    }
  }

  function addConnection(connection: CanvasConnection) {
    if (!currentWorkflow.value) return
    currentWorkflow.value.connections.push(connection)
    isDirty.value = true
  }

  function removeConnection(connectionId: string) {
    if (!currentWorkflow.value) return
    const index = currentWorkflow.value.connections.findIndex(conn => conn.id === connectionId)
    if (index !== -1) {
      currentWorkflow.value.connections.splice(index, 1)
      isDirty.value = true
    }
  }

  return {
    // State
    currentWorkflow,
    workflows,
    activeNodeId,
    selectedNodeIds,
    isDirty,
    
    // Getters
    hasActiveWorkflow,
    activeNode,
    
    // Actions
    setCurrentWorkflow,
    setActiveNode,
    setSelectedNodes,
    addNode,
    removeNode,
    updateNode,
    addConnection,
    removeConnection
  }
})