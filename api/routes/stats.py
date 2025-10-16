"""
Statistics and Performance Endpoints

Uses existing manager.get_state() for metrics.
"""

from fastapi import APIRouter
import logging

from ..types import PerformanceStats
from core.inference_service import get_inference_service

logger = logging.getLogger(__name__)

router = APIRouter()


@router.get("/stats", response_model=PerformanceStats)
async def get_stats():
    """
    Get performance statistics.
    Uses existing manager.get_state()
    """
    service = get_inference_service()
    manager = service.get_active_manager()
    
    if not manager or not hasattr(manager, 'get_state'):
        return PerformanceStats()
    
    state = manager.get_state()
    if not state:
        return PerformanceStats()
    
    return PerformanceStats(
        time_to_first_token=state.get("time_to_first_token"),
        tokens_per_second=state.get("tokens_per_second"),
        input_tokens=state.get("input_tokens"),
        output_tokens=state.get("output_tokens"),
        total_time=state.get("total_time")
    )

