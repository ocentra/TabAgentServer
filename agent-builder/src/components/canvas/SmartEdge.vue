<template>
  <path
    :id="id"
    :class="['smart-edge', { selected, animated }]"
    :d="edgePath"
    :style="edgeStyle"
    :marker-end="markerEnd"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { EdgeProps } from '@vue-flow/core'
import { getSmartEdgePath } from '@/utils/edge-routing'

interface Props extends EdgeProps {
  id: string
  sourceX: number
  sourceY: number
  targetX: number
  targetY: number
  sourcePosition: any
  targetPosition: any
  selected?: boolean
  animated?: boolean
  style?: Record<string, any>
  markerEnd?: string
}

const props = defineProps<Props>()

// Use smart routing that avoids nodes!
const edgePath = computed(() => {
  return getSmartEdgePath({
    sourceX: props.sourceX,
    sourceY: props.sourceY,
    targetX: props.targetX,
    targetY: props.targetY,
    sourcePosition: props.sourcePosition,
    targetPosition: props.targetPosition
  })
})

const edgeStyle = computed(() => ({
  strokeWidth: props.selected ? 3 : 2,
  stroke: props.style?.stroke || 'var(--color--foreground, #999)',
  strokeDasharray: props.style?.strokeDasharray,
  ...props.style
}))
</script>

<style scoped>
.smart-edge {
  fill: none;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: stroke 0.2s ease, stroke-width 0.2s ease;
  pointer-events: stroke;
  cursor: pointer;
}

.smart-edge.selected {
  stroke: #ff6d5a !important;
  stroke-width: 3px;
}

.smart-edge.animated {
  stroke-dasharray: 5;
  animation: dashdraw 0.5s linear infinite;
}

@keyframes dashdraw {
  from {
    stroke-dashoffset: 10;
  }
  to {
    stroke-dashoffset: 0;
  }
}

.smart-edge:hover {
  stroke-width: 3px;
}
</style>

