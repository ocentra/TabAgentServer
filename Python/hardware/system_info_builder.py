"""
System Information Builder

Shared logic for building system info responses.
Used by BOTH HTTP API and native messaging (DRY principle).
"""

import logging
from typing import Dict, Any, List, Optional

from Python.core.message_types import HardwareInfo
from Python.hardware.engine_detection import DeviceEngineMapper

logger = logging.getLogger(__name__)


class SystemInfoFields:
    """Field names for system info (no string literals)"""
    OS_VERSION = "os_version"
    PROCESSOR = "processor"
    PHYSICAL_MEMORY = "physical_memory"
    DEVICES = "devices"
    PYTHON_PACKAGES = "python_packages"


class DeviceFields:
    """Field names for device info (no string literals)"""
    NAME = "name"
    AVAILABLE = "available"
    CORES = "cores"
    THREADS = "threads"
    MAX_CLOCK_SPEED_MHZ = "max_clock_speed_mhz"
    VRAM_GB = "vram_gb"
    DRIVER_VERSION = "driver_version"
    POWER_MODE = "power_mode"
    INFERENCE_ENGINES = "inference_engines"
    ERROR = "error"


class DeviceTypeNames:
    """Device type names (no string literals)"""
    CPU = "cpu"
    NVIDIA_DGPU = "nvidia_dgpu"
    AMD_DGPU = "amd_dgpu"
    AMD_IGPU = "amd_igpu"
    INTEL_GPU = "intel_gpu"
    NPU = "npu"


def build_system_info_dict(
    hardware_info: HardwareInfo,
    verbose: bool = False
) -> Dict[str, Any]:
    """
    Build complete system information dictionary.
    
    Shared function used by both HTTP API and native messaging.
    
    Args:
        hardware_info: Hardware information from HardwareDetector
        verbose: Include Python packages if True
        
    Returns:
        System info dictionary
    """
    # Map engines to devices
    engine_mapper = DeviceEngineMapper()
    device_mappings = engine_mapper.map_engines_to_devices(hardware_info)
    
    # Build basic system info
    system_info: Dict[str, Any] = {
        SystemInfoFields.OS_VERSION: hardware_info.os_version,
        SystemInfoFields.PROCESSOR: _get_processor_name(hardware_info),
    }
    
    # Build devices dictionary
    devices_dict: Dict[str, Any] = {}
    
    # CPU device
    if hardware_info.cpu and hardware_info.cpu.available:
        cpu_engines = _find_device_engines(device_mappings, DeviceTypeNames.CPU)
        devices_dict[DeviceTypeNames.CPU] = _build_cpu_info(hardware_info.cpu, cpu_engines)
    
    # NVIDIA GPUs
    nvidia_gpus = []
    for gpu in hardware_info.nvidia_gpus:
        if gpu.available:
            gpu_engines = _find_device_engines_by_name(
                device_mappings,
                DeviceTypeNames.NVIDIA_DGPU,
                gpu.name
            )
            nvidia_gpus.append(_build_gpu_info(gpu, gpu_engines))
    
    if nvidia_gpus:
        devices_dict[DeviceTypeNames.NVIDIA_DGPU] = nvidia_gpus
    
    # AMD GPUs
    amd_gpus = []
    for gpu in hardware_info.amd_gpus:
        if gpu.available:
            gpu_engines = _find_device_engines_by_name(
                device_mappings,
                DeviceTypeNames.AMD_DGPU,
                gpu.name
            )
            amd_gpus.append(_build_gpu_info(gpu, gpu_engines))
    
    if amd_gpus:
        devices_dict[DeviceTypeNames.AMD_DGPU] = amd_gpus
    
    # Intel GPUs
    intel_gpus = []
    for gpu in hardware_info.intel_gpus:
        if gpu.available:
            gpu_engines = _find_device_engines_by_name(
                device_mappings,
                DeviceTypeNames.INTEL_GPU,
                gpu.name
            )
            intel_gpus.append(_build_gpu_info(gpu, gpu_engines))
    
    if intel_gpus:
        devices_dict[DeviceTypeNames.INTEL_GPU] = intel_gpus
    
    # NPU device
    if hardware_info.npu and hardware_info.npu.available:
        npu_engines = _find_device_engines(device_mappings, DeviceTypeNames.NPU)
        devices_dict[DeviceTypeNames.NPU] = _build_npu_info(hardware_info.npu, npu_engines)
    
    system_info[SystemInfoFields.DEVICES] = devices_dict
    
    # Add Python packages in verbose mode
    if verbose:
        system_info[SystemInfoFields.PYTHON_PACKAGES] = _get_python_packages()
    
    return system_info


def _get_processor_name(hardware_info: HardwareInfo) -> str:
    """Extract processor name"""
    if hardware_info.cpu and hardware_info.cpu.available:
        return hardware_info.cpu.name
    return "Unknown"


def _find_device_engines(
    device_mappings: List[Any],
    device_type: str
) -> Optional[Dict[str, Dict[str, Any]]]:
    """Find engine mapping for device type"""
    for mapping in device_mappings:
        if mapping.device_type == device_type:
            return mapping.engines
    return None


def _find_device_engines_by_name(
    device_mappings: List[Any],
    device_type: str,
    device_name: str
) -> Optional[Dict[str, Dict[str, Any]]]:
    """Find engine mapping for specific device by name"""
    for mapping in device_mappings:
        if mapping.device_type == device_type and mapping.device_name == device_name:
            return mapping.engines
    return None


def _build_cpu_info(cpu_info: Any, engines: Optional[Dict[str, Dict[str, Any]]]) -> Dict[str, Any]:
    """Build CPU info dictionary"""
    info = {
        DeviceFields.NAME: cpu_info.name,
        DeviceFields.AVAILABLE: cpu_info.available,
    }
    
    if hasattr(cpu_info, 'cores') and cpu_info.cores:
        info[DeviceFields.CORES] = cpu_info.cores
    
    if hasattr(cpu_info, 'threads') and cpu_info.threads:
        info[DeviceFields.THREADS] = cpu_info.threads
    
    if hasattr(cpu_info, 'max_clock_speed_mhz') and cpu_info.max_clock_speed_mhz:
        info[DeviceFields.MAX_CLOCK_SPEED_MHZ] = cpu_info.max_clock_speed_mhz
    
    if engines:
        info[DeviceFields.INFERENCE_ENGINES] = engines
    
    return info


def _build_gpu_info(gpu_info: Any, engines: Optional[Dict[str, Dict[str, Any]]]) -> Dict[str, Any]:
    """Build GPU info dictionary"""
    info = {
        DeviceFields.NAME: gpu_info.name,
        DeviceFields.AVAILABLE: gpu_info.available,
    }
    
    if hasattr(gpu_info, 'vram_gb') and gpu_info.vram_gb:
        info[DeviceFields.VRAM_GB] = gpu_info.vram_gb
    
    if hasattr(gpu_info, 'driver_version') and gpu_info.driver_version:
        info[DeviceFields.DRIVER_VERSION] = gpu_info.driver_version
    
    if engines:
        info[DeviceFields.INFERENCE_ENGINES] = engines
    
    return info


def _build_npu_info(npu_info: Any, engines: Optional[Dict[str, Dict[str, Any]]]) -> Dict[str, Any]:
    """Build NPU info dictionary"""
    info = {
        DeviceFields.NAME: npu_info.name,
        DeviceFields.AVAILABLE: npu_info.available,
    }
    
    if hasattr(npu_info, 'driver_version') and npu_info.driver_version:
        info[DeviceFields.DRIVER_VERSION] = npu_info.driver_version
    
    if hasattr(npu_info, 'power_mode') and npu_info.power_mode:
        info[DeviceFields.POWER_MODE] = npu_info.power_mode
    
    if engines:
        info[DeviceFields.INFERENCE_ENGINES] = engines
    
    return info


def _get_python_packages() -> List[str]:
    """Get installed Python packages"""
    try:
        import importlib.metadata
        
        distributions = importlib.metadata.distributions()
        packages = [
            f"{dist.metadata['name']}=={dist.metadata['version']}"
            for dist in distributions
        ]
        return sorted(packages)
    
    except Exception as e:
        logger.warning(f"Could not get Python packages: {e}")
        return []

