// Vue Flow composable for basic setup and configuration
import { ref, computed, nextTick } from 'vue'
import { useVueFlow as useVueFlowCore } from '@vue-flow/core'
import dagre from '@dagrejs/dagre'
import type { 
  CanvasNode, 
  CanvasConnection, 
  DagreLayoutOptions,
  LayoutedElements
} from '@/types/vue-flow'

export function useVueFlow() {
  // Get Vue Flow instance
  const { 
    nodes, 
    edges, 
    viewport,
    addNodes,
    addEdges,
    removeNodes,
    removeEdges,
    updateNode,
    updateEdge,
    findNode,
    findEdge,
    getNodes,
    getEdges,
    setViewport,
    fitView,
    zoomIn,
    zoomOut,
    zoomTo,
    project,
    vueFlowRef
  } = useVueFlowCore()

  // Canvas state
  const isInitialized = ref(false)
  const selectedNodeIds = ref<string[]>([])
  const selectedEdgeIds = ref<string[]>([])
  const isDragging = ref(false)
  const isConnecting = ref(false)

  // Default Vue Flow options
  const defaultOptions = {
    pannable: true,
    zoomable: true,
    selectable: true,
    draggable: true,
    connectable: true,
    snapToGrid: false,
    snapGrid: [15, 15] as [number, number],
    onlyRenderVisibleElements: true,
    minZoom: 0.1,
    maxZoom: 4,
    defaultZoom: 1,
    defaultPosition: [0, 0] as [number, number],
    fitViewOnInit: true,
    nodesConnectable: true,
    nodesDraggable: true,
    edgesUpdatable: true,
    elementsSelectable: true,
    selectNodesOnDrag: true,
    multiSelectionKeyCode: 'Meta',
    deleteKeyCode: 'Delete',
    selectionKeyCode: 'Shift',
    zoomActivationKeyCode: 'Meta',
    panActivationKeyCode: 'Space'
  }

  // Initialize Vue Flow
  const initialize = async () => {
    if (isInitialized.value) return
    
    await nextTick()
    isInitialized.value = true
  }

  // Auto-layout using Dagre
  const getLayoutedElements = (
    nodes: CanvasNode[], 
    edges: CanvasConnection[], 
    options: DagreLayoutOptions = { direction: 'TB' }
  ): LayoutedElements => {
    const dagreGraph = new dagre.graphlib.Graph()
    dagreGraph.setDefaultEdgeLabel(() => ({}))
    
    const {
      direction = 'TB',
      nodeWidth = 172,
      nodeHeight = 80,
      rankSep = 50,
      nodeSep = 50,
      edgeSep = 10,
      ranker = 'network-simplex'
    } = options

    dagreGraph.setGraph({ 
      rankdir: direction,
      ranksep: rankSep,
      nodesep: nodeSep,
      edgesep: edgeSep,
      ranker
    })

    // Add nodes to dagre graph
    nodes.forEach((node) => {
      dagreGraph.setNode(node.id, { 
        width: node.dimensions?.width || nodeWidth, 
        height: node.dimensions?.height || nodeHeight 
      })
    })

    // Add edges to dagre graph
    edges.forEach((edge) => {
      dagreGraph.setEdge(edge.source, edge.target)
    })

    // Calculate layout
    dagre.layout(dagreGraph)

    // Apply layout to nodes
    const layoutedNodes = nodes.map((node) => {
      const nodeWithPosition = dagreGraph.node(node.id)
      return {
        ...node,
        position: {
          x: nodeWithPosition.x - (node.dimensions?.width || nodeWidth) / 2,
          y: nodeWithPosition.y - (node.dimensions?.height || nodeHeight) / 2
        }
      }
    })

    return { nodes: layoutedNodes, edges }
  }

  // Apply auto-layout to current nodes and edges
  const applyAutoLayout = async (options?: DagreLayoutOptions) => {
    const currentNodes = getNodes.value as CanvasNode[]
    const currentEdges = getEdges.value as CanvasConnection[]
    
    if (currentNodes.length === 0) return

    const { nodes: layoutedNodes } = getLayoutedElements(
      currentNodes, 
      currentEdges, 
      options
    )

    // Update nodes with new positions
    layoutedNodes.forEach((node) => {
      updateNode(node.id, { position: node.position })
    })

    await nextTick()
    await fitView({ duration: 300 })
  }

  // Node selection helpers
  const selectNode = (nodeId: string, addToSelection = false) => {
    if (addToSelection) {
      if (!selectedNodeIds.value.includes(nodeId)) {
        selectedNodeIds.value.push(nodeId)
      }
    } else {
      selectedNodeIds.value = [nodeId]
    }
  }

  const deselectNode = (nodeId: string) => {
    selectedNodeIds.value = selectedNodeIds.value.filter(id => id !== nodeId)
  }

  const clearSelection = () => {
    selectedNodeIds.value = []
    selectedEdgeIds.value = []
  }

  // Canvas helpers
  const centerCanvas = async () => {
    await fitView({ duration: 300 })
  }

  const resetZoom = async () => {
    await zoomTo(1, { duration: 300 })
  }

  // Computed properties
  const selectedNodes = computed(() => 
    selectedNodeIds.value.map(id => findNode(id)).filter(Boolean)
  )

  const hasSelection = computed(() => 
    selectedNodeIds.value.length > 0 || selectedEdgeIds.value.length > 0
  )

  const canvasState = computed(() => ({
    isInitialized: isInitialized.value,
    isDragging: isDragging.value,
    isConnecting: isConnecting.value,
    hasSelection: hasSelection.value,
    selectedCount: selectedNodeIds.value.length + selectedEdgeIds.value.length
  }))

  return {
    // Vue Flow core
    nodes,
    edges,
    viewport,
    addNodes,
    addEdges,
    removeNodes,
    removeEdges,
    updateNode,
    updateEdge,
    findNode,
    findEdge,
    getNodes,
    getEdges,
    setViewport,
    fitView,
    zoomIn,
    zoomOut,
    zoomTo,
    project,
    vueFlowRef,

    // Configuration
    defaultOptions,
    initialize,

    // Layout
    getLayoutedElements,
    applyAutoLayout,

    // Selection
    selectedNodeIds,
    selectedEdgeIds,
    selectedNodes,
    selectNode,
    deselectNode,
    clearSelection,

    // Canvas helpers
    centerCanvas,
    resetZoom,

    // State
    isInitialized,
    isDragging,
    isConnecting,
    hasSelection,
    canvasState
  }
}