<template>
  <div class="workflow-canvas" ref="canvasContainer">
    <div class="canvas-wrapper">
      <div 
        class="jsplumb-container" 
        ref="jsPlumbContainer"
        @mousedown="onCanvasMouseDown"
        @mousemove="onCanvasMouseMove"
        @mouseup="onCanvasMouseUp"
      >
        <!-- Nodes -->
        <div
          v-for="node in nodes"
          :key="node.id"
          :id="node.id"
          class="node-wrapper"
          :style="nodePosition(node)"
          @mousedown="onNodeMouseDown($event, node)"
        >
          <NodeRenderer :node="node" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { jsPlumb } from 'jsplumb'
import NodeRenderer from './NodeRenderer.vue'
import type { INodeUi } from '@/Interface'

interface Props {
  nodes?: INodeUi[]
  readOnly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  nodes: () => [],
  readOnly: false
})

const canvasContainer = ref<HTMLElement>()
const jsPlumbContainer = ref<HTMLElement>()
let jsPlumbInstance: any = null

const nodePosition = computed(() => (node: INodeUi) => ({
  left: `${node.position[0]}px`,
  top: `${node.position[1]}px`,
  position: 'absolute'
}))

function initJsPlumb() {
  if (!jsPlumbContainer.value) return
  
  jsPlumbInstance = jsPlumb.newInstance({
    container: jsPlumbContainer.value,
    connector: ['Bezier', { curviness: 60 }],
    paintStyle: { stroke: '#666', strokeWidth: 2 },
    hoverPaintStyle: { stroke: '#ff6d5a', strokeWidth: 3 },
    endpoint: ['Dot', { radius: 5 }],
    endpointStyle: { fill: '#666' },
    endpointHoverStyle: { fill: '#ff6d5a' },
    anchors: ['Right', 'Left']
  })
}

function onCanvasMouseDown(event: MouseEvent) {
  // Handle canvas interactions
}

function onCanvasMouseMove(event: MouseEvent) {
  // Handle canvas mouse move
}

function onCanvasMouseUp(event: MouseEvent) {
  // Handle canvas mouse up
}

function onNodeMouseDown(event: MouseEvent, node: INodeUi) {
  // Handle node interactions
  event.stopPropagation()
}

onMounted(() => {
  initJsPlumb()
})

onUnmounted(() => {
  if (jsPlumbInstance) {
    jsPlumbInstance.destroy()
  }
})
</script>

<style lang="scss" scoped>
.workflow-canvas {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  background: #f9f9f9;
}

.canvas-wrapper {
  width: 100%;
  height: 100%;
  position: relative;
}

.jsplumb-container {
  width: 100%;
  height: 100%;
  position: relative;
}

.node-wrapper {
  z-index: 10;
}
</style>