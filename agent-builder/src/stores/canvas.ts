import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ViewportTransform } from '@vue-flow/core'

export const useCanvasStore = defineStore('canvas', () => {
  // State
  const viewport = ref<ViewportTransform>({ x: 0, y: 0, zoom: 1 })
  const zoom = ref(1)
  const pannable = ref(true)
  const selectable = ref(true)
  const connecting = ref(false)
  const draggedNodeType = ref<string | null>(null)

  // Getters
  const isConnecting = computed(() => connecting.value)
  const isDraggingNode = computed(() => draggedNodeType.value !== null)

  // Actions
  function setViewport(newViewport: ViewportTransform) {
    viewport.value = newViewport
    zoom.value = newViewport.zoom
  }

  function setZoom(newZoom: number) {
    zoom.value = newZoom
    viewport.value = { ...viewport.value, zoom: newZoom }
  }

  function setPannable(value: boolean) {
    pannable.value = value
  }

  function setSelectable(value: boolean) {
    selectable.value = value
  }

  function setConnecting(value: boolean) {
    connecting.value = value
  }

  function setDraggedNodeType(nodeType: string | null) {
    draggedNodeType.value = nodeType
  }

  function fitView() {
    // This will be implemented when we have the Vue Flow instance
    console.log('Fit view requested')
  }

  function zoomIn() {
    const newZoom = Math.min(zoom.value * 1.2, 3)
    setZoom(newZoom)
  }

  function zoomOut() {
    const newZoom = Math.max(zoom.value / 1.2, 0.1)
    setZoom(newZoom)
  }

  function resetZoom() {
    setZoom(1)
  }

  return {
    // State
    viewport,
    zoom,
    pannable,
    selectable,
    connecting,
    draggedNodeType,
    
    // Getters
    isConnecting,
    isDraggingNode,
    
    // Actions
    setViewport,
    setZoom,
    setPannable,
    setSelectable,
    setConnecting,
    setDraggedNodeType,
    fitView,
    zoomIn,
    zoomOut,
    resetZoom
  }
})