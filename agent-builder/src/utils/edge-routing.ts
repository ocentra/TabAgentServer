// Stolen from n8n - intelligent edge routing to avoid node overlaps!
import { getBezierPath, getSmoothStepPath, Position } from '@vue-flow/core'
import type { EdgeProps } from '@vue-flow/core'

const EDGE_PADDING_BOTTOM = 130
const EDGE_PADDING_X = 40
const EDGE_BORDER_RADIUS = 16
const HANDLE_SIZE = 20

const isRightOfSourceHandle = (sourceX: number, targetX: number) => 
  sourceX - HANDLE_SIZE > targetX

export function getSmartEdgePath(props: Pick<
  EdgeProps,
  'sourceX' | 'sourceY' | 'sourcePosition' | 'targetX' | 'targetY' | 'targetPosition'
>) {
  const { targetX, targetY, sourceX, sourceY, sourcePosition, targetPosition } = props
  
  // If target is to the right of source, use simple Bezier
  if (!isRightOfSourceHandle(sourceX, targetX)) {
    return getBezierPath(props)
  }

  // Connection is backwards - need to avoid overlapping the source node!
  // Create a path that goes DOWN and AROUND
  const firstSegmentTargetX = (sourceX + targetX) / 2
  const firstSegmentTargetY = sourceY + EDGE_PADDING_BOTTOM

  const firstSegment = getSmoothStepPath({
    sourceX,
    sourceY,
    targetX: firstSegmentTargetX,
    targetY: firstSegmentTargetY,
    sourcePosition,
    targetPosition: Position.Right,
    borderRadius: EDGE_BORDER_RADIUS,
    offset: EDGE_PADDING_X,
  })

  const secondSegment = getSmoothStepPath({
    sourceX: firstSegmentTargetX,
    sourceY: firstSegmentTargetY,
    targetX,
    targetY,
    sourcePosition: Position.Left,
    targetPosition,
    borderRadius: EDGE_BORDER_RADIUS,
    offset: EDGE_PADDING_X,
  })

  // Combine the two segments for smart routing!
  return [firstSegment[0], secondSegment[0]].join(' ')
}

