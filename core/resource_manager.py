"""
Resource Manager
================

Tracks and manages VRAM/RAM allocation for multi-model loading.
Critical for agentic systems where multiple models run simultaneously.

Responsibilities:
- Query available VRAM/RAM
- Estimate model memory requirements
- Suggest offload strategies
- Track allocations per model
- Prevent over-allocation
"""

import logging
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from pathlib import Path
from enum import Enum

logger = logging.getLogger(__name__)


class OffloadStrategy(str, Enum):
    """Model offload strategies"""
    AUTO = "auto"               # Let server decide best strategy
    FULL_VRAM = "full_vram"     # All layers on GPU (fastest, most VRAM)
    FULL_RAM = "full_ram"       # All layers on CPU RAM (slowest, no VRAM)
    HYBRID = "hybrid"           # Split layers between VRAM and RAM
    FAIL_IF_INSUFFICIENT = "fail"  # Fail if can't fit in VRAM


@dataclass
class ResourceStatus:
    """Current resource availability"""
    vram_total_mb: int
    vram_used_mb: int
    vram_available_mb: int
    ram_total_mb: int
    ram_used_mb: int
    ram_available_mb: int
    gpu_count: int
    
    def to_dict(self) -> Dict:
        """Convert to dictionary"""
        return {
            "vram": {
                "total_mb": self.vram_total_mb,
                "used_mb": self.vram_used_mb,
                "available_mb": self.vram_available_mb,
            },
            "ram": {
                "total_mb": self.ram_total_mb,
                "used_mb": self.ram_used_mb,
                "available_mb": self.ram_available_mb,
            },
            "gpu_count": self.gpu_count
        }


@dataclass
class ModelResourceEstimate:
    """Estimated resource requirements for a model"""
    total_size_mb: int          # Total model size
    vram_required_mb: int       # VRAM needed (full GPU)
    ram_required_mb: int        # RAM needed (full CPU)
    min_vram_mb: int            # Minimum VRAM (partial offload)
    layer_count: Optional[int] = None
    mb_per_layer: Optional[float] = None


@dataclass
class OffloadOption:
    """A possible offload configuration"""
    strategy: str               # "full_vram", "hybrid", "full_ram"
    vram_layers: int
    ram_layers: int
    vram_mb: int
    ram_mb: int
    speed_rating: str           # "fast", "medium", "slow"
    description: str


@dataclass
class ModelAllocation:
    """Resource allocation for a loaded model"""
    model_id: str
    vram_mb: int
    ram_mb: int
    backend: str
    vram_layers: int = 0
    ram_layers: int = 0


class ResourceManager:
    """
    Manages VRAM and RAM allocation for models.
    
    Tracks what's allocated and provides smart suggestions.
    """
    
    def __init__(self):
        """Initialize resource manager"""
        self._allocations: Dict[str, ModelAllocation] = {}
        logger.info("ResourceManager initialized")
    
    def get_resource_status(self) -> ResourceStatus:
        """
        Get current resource status.
        
        Returns:
            ResourceStatus with VRAM/RAM info
        """
        from hardware.hardware_detection import HardwareDetector
        import psutil
        
        detector = HardwareDetector()
        
        # Get VRAM info
        vram_total_mb = 0
        gpu_count = 0
        
        try:
            vram_list = detector.get_nvidia_vram_mb()
            if vram_list:
                vram_total_mb = sum(vram_list)
                gpu_count = len(vram_list)
        except Exception as e:
            logger.debug(f"Could not get NVIDIA VRAM: {e}")
        
        # Calculate used VRAM from allocations
        vram_used_mb = sum(alloc.vram_mb for alloc in self._allocations.values())
        vram_available_mb = max(0, vram_total_mb - vram_used_mb)
        
        # Get RAM info
        ram = psutil.virtual_memory()
        ram_total_mb = ram.total // (1024 * 1024)
        ram_available_mb = ram.available // (1024 * 1024)
        ram_used_mb = ram_total_mb - ram_available_mb
        
        return ResourceStatus(
            vram_total_mb=vram_total_mb,
            vram_used_mb=vram_used_mb,
            vram_available_mb=vram_available_mb,
            ram_total_mb=ram_total_mb,
            ram_used_mb=ram_used_mb,
            ram_available_mb=ram_available_mb,
            gpu_count=gpu_count
        )
    
    def estimate_model_size(self, model_path: str) -> ModelResourceEstimate:
        """
        Estimate memory requirements for a model.
        
        Args:
            model_path: Path to model file
            
        Returns:
            ModelResourceEstimate with size and layer info
        """
        path = Path(model_path)
        
        # Get file size
        if not path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")
        
        file_size_mb = path.stat().st_size // (1024 * 1024)
        
        # Estimate runtime memory (typically 1.2-1.5x file size)
        # Account for: model weights + KV cache + activations
        runtime_multiplier = 1.5
        estimated_total = int(file_size_mb * runtime_multiplier)
        
        # Estimate layers (rough heuristic)
        # Small models (~1-3B params): 24-32 layers
        # Medium models (~7-13B params): 32-40 layers
        # Large models (13B+ params): 40-80 layers
        if file_size_mb < 2000:  # < 2GB
            estimated_layers = 28
        elif file_size_mb < 8000:  # < 8GB
            estimated_layers = 32
        else:
            estimated_layers = 40
        
        mb_per_layer = estimated_total / estimated_layers if estimated_layers > 0 else 0
        
        return ModelResourceEstimate(
            total_size_mb=estimated_total,
            vram_required_mb=estimated_total,  # Full GPU
            ram_required_mb=estimated_total,   # Full CPU
            min_vram_mb=int(estimated_total * 0.3),  # Minimum: 30% on GPU
            layer_count=estimated_layers,
            mb_per_layer=mb_per_layer
        )
    
    def suggest_offload_strategies(
        self,
        model_estimate: ModelResourceEstimate,
        available_vram_mb: int,
        available_ram_mb: int
    ) -> List[OffloadOption]:
        """
        Suggest offload strategies based on available resources.
        
        Args:
            model_estimate: Model size estimate
            available_vram_mb: Available VRAM
            available_ram_mb: Available RAM
            
        Returns:
            List of viable offload options
        """
        options: List[OffloadOption] = []
        total_layers = model_estimate.layer_count or 32
        mb_per_layer = model_estimate.mb_per_layer or (model_estimate.total_size_mb / total_layers)
        
        # Option 1: Full VRAM (if fits)
        if available_vram_mb >= model_estimate.vram_required_mb:
            options.append(OffloadOption(
                strategy="full_vram",
                vram_layers=total_layers,
                ram_layers=0,
                vram_mb=model_estimate.vram_required_mb,
                ram_mb=0,
                speed_rating="fast",
                description=f"All {total_layers} layers on GPU. Fastest inference."
            ))
        
        # Option 2: Hybrid (partial VRAM)
        if available_vram_mb >= model_estimate.min_vram_mb and available_ram_mb >= model_estimate.ram_required_mb:
            # Calculate how many layers fit in VRAM
            vram_layers = int(available_vram_mb / mb_per_layer) if mb_per_layer > 0 else 0
            vram_layers = min(vram_layers, total_layers)
            ram_layers = total_layers - vram_layers
            
            if vram_layers > 0 and ram_layers > 0:
                vram_mb = int(vram_layers * mb_per_layer)
                ram_mb = int(ram_layers * mb_per_layer)
                
                options.append(OffloadOption(
                    strategy="hybrid",
                    vram_layers=vram_layers,
                    ram_layers=ram_layers,
                    vram_mb=vram_mb,
                    ram_mb=ram_mb,
                    speed_rating="medium",
                    description=f"{vram_layers} layers GPU, {ram_layers} layers RAM. Balanced."
                ))
        
        # Option 3: Full RAM (always possible if enough RAM)
        if available_ram_mb >= model_estimate.ram_required_mb:
            options.append(OffloadOption(
                strategy="full_ram",
                vram_layers=0,
                ram_layers=total_layers,
                vram_mb=0,
                ram_mb=model_estimate.ram_required_mb,
                speed_rating="slow",
                description=f"All {total_layers} layers on RAM. Slower but works."
            ))
        
        return options
    
    def allocate(self, allocation: ModelAllocation) -> None:
        """
        Record resource allocation for a model.
        
        Args:
            allocation: Model allocation info
        """
        self._allocations[allocation.model_id] = allocation
        logger.info(f"Allocated resources for {allocation.model_id}: VRAM={allocation.vram_mb}MB, RAM={allocation.ram_mb}MB")
    
    def deallocate(self, model_id: str) -> bool:
        """
        Release resource allocation for a model.
        
        Args:
            model_id: Model identifier
            
        Returns:
            True if deallocated, False if not found
        """
        if model_id in self._allocations:
            alloc = self._allocations.pop(model_id)
            logger.info(f"Deallocated resources for {model_id}: VRAM={alloc.vram_mb}MB, RAM={alloc.ram_mb}MB")
            return True
        return False
    
    def get_allocation(self, model_id: str) -> Optional[ModelAllocation]:
        """Get allocation for a model"""
        return self._allocations.get(model_id)
    
    def get_all_allocations(self) -> Dict[str, ModelAllocation]:
        """Get all current allocations"""
        return self._allocations.copy()


# Global singleton
_resource_manager: Optional[ResourceManager] = None


def get_resource_manager() -> ResourceManager:
    """Get global resource manager instance"""
    global _resource_manager
    if _resource_manager is None:
        _resource_manager = ResourceManager()
    return _resource_manager

