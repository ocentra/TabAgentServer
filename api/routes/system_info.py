"""
System Information Endpoint

Provides comprehensive hardware and inference engine information.
Uses shared system_info_builder (DRY principle).
"""

import logging
from typing import Dict, Any
from fastapi import APIRouter, Query

from hardware.hardware_detection import get_hardware_detector
from hardware.system_info_builder import build_system_info_dict

logger = logging.getLogger(__name__)

router = APIRouter()


@router.get("/system-info")
async def get_system_info(
    verbose: bool = Query(False, description="Include verbose information (Python packages)")
) -> Dict[str, Any]:
    """
    Get comprehensive system information.
    
    Returns hardware details and inference engine availability per device.
    Uses shared system_info_builder (DRY - reused by native messaging).
    
    Args:
        verbose: If True, include Python package versions
        
    Returns:
        System information dictionary
    """
    try:
        # Get hardware info
        hardware_detector = get_hardware_detector()
        hardware_info = hardware_detector.get_hardware_info()
        
        # Use shared builder (DRY principle)
        system_info = build_system_info_dict(hardware_info, verbose)
        
        logger.info("System info retrieved successfully")
        return system_info
    
    except Exception as e:
        logger.error(f"Error getting system info: {e}", exc_info=True)
        raise

