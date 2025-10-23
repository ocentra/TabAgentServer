"""
Hardware detection and capability analysis for TabAgent.

This module provides OS-specific hardware detection including:
- CPU information
- GPU detection (NVIDIA, AMD, Intel)
- VRAM detection via nvidia-smi
- Hardware acceleration capabilities
"""

import platform
import subprocess
import logging
from abc import ABC, abstractmethod
from typing import List, Optional, Tuple
from enum import Enum

from Python.core.message_types import (
    CPUInfo,
    GPUInfo,
    NPUInfo,
    HardwareInfo,
    HardwareCapabilities,
    GPUVendor,
    GPUType,
    AccelerationBackend,
)


logger = logging.getLogger(__name__)



# These are used for keyword-based GPU classification


class NVIDIAGPUKeyword(str, Enum):
    """NVIDIA discrete GPU identification keywords"""
    GEFORCE = "geforce"
    RTX = "rtx"
    GTX = "gtx"
    QUADRO = "quadro"
    TESLA = "tesla"
    TITAN = "titan"
    A100 = "a100"
    A40 = "a40"
    A30 = "a30"
    A10 = "a10"
    A6000 = "a6000"
    A5000 = "a5000"
    A4000 = "a4000"
    A2000 = "a2000"
    L40 = "l40"
    L4 = "l4"
    T1000 = "t1000"
    T600 = "t600"
    T400 = "t400"


class AMDGPUKeyword(str, Enum):
    """AMD discrete GPU identification keywords"""
    RX = "rx "  # Space is intentional for "RX 5700" pattern
    XT = "xt"
    PRO_W = "pro w"
    PRO_V = "pro v"
    RADEON_PRO = "radeon pro"
    FIREPRO = "firepro"
    FURY = "fury"
    VEGA = "vega"
    NAVI = "navi"
    RDNA = "rdna"


class VRAMDetectionMethod(str, Enum):
    """Methods for VRAM detection"""
    NVIDIA_SMI = "nvidia-smi"
    WMI = "wmi"
    DXDIAG = "dxdiag"
    ROCM_SMI = "rocm-smi"
    LSPCI = "lspci"
    SYSTEM_PROFILER = "system_profiler"


class OSType(str, Enum):
    """Supported operating system types"""
    WINDOWS = "Windows"
    LINUX = "Linux"
    DARWIN = "Darwin"  # macOS
    UNKNOWN = "Unknown"


# Constants for command execution
class CommandTimeout(int, Enum):
    """Timeout values for external commands (seconds)"""
    NVIDIA_SMI = 5
    DXDIAG = 30
    WMI = 10
    ROCM_SMI = 5
    LSPCI = 5


class VRAMUnit(str, Enum):
    """VRAM measurement units"""
    BYTES = "bytes"
    MEGABYTES = "mb"
    GIGABYTES = "gb"


# Conversion constants
BYTES_PER_GB = 1024 ** 3
BYTES_PER_MB = 1024 ** 2
MB_PER_GB = 1024


class HardwareDetector(ABC):
    """
    Abstract base class for OS-specific hardware detection.
    
    Provides interface for detecting:
    - CPU information (name, cores, threads)
    - GPU devices (NVIDIA, AMD, Intel)
    - VRAM amounts
    - Hardware acceleration capabilities
    """
    
    def __init__(self):
        """Initialize hardware detector with OS-specific implementations"""
        self.os_type: OSType = self._detect_os()
        logger.info(f"HardwareDetector initialized for {self.os_type.value}")
    
    @staticmethod
    def _detect_os() -> OSType:
        """
        Detect operating system type.
        
        Returns:
            OSType enum value for current OS
        """
        system = platform.system()
        try:
            return OSType(system)
        except ValueError:
            logger.warning(f"Unknown OS type: {system}, defaulting to UNKNOWN")
            return OSType.UNKNOWN
    
    def get_hardware_info(self) -> HardwareInfo:
        """
        Get complete hardware information.
        
        Returns:
            HardwareInfo object with all detected hardware
        """
        cpu_info = self.get_cpu_info()
        nvidia_gpus = self.get_nvidia_gpus()
        amd_gpus = self.get_amd_gpus()
        intel_gpus = self.get_intel_gpus()
        npu_info = self.get_npu_info()
        capabilities = self.get_capabilities()
        os_version = self.get_os_version()
        
        return HardwareInfo(
            cpu=cpu_info,
            nvidia_gpus=nvidia_gpus,
            amd_gpus=amd_gpus,
            intel_gpus=intel_gpus,
            npu=npu_info,
            capabilities=capabilities,
            os_version=os_version
        )
    
    @abstractmethod
    def get_cpu_info(self) -> CPUInfo:
        """
        Get CPU information.
        
        Returns:
            CPUInfo object with CPU details
        """
        pass
    
    @abstractmethod
    def get_nvidia_gpus(self) -> List[GPUInfo]:
        """
        Get NVIDIA GPU information.
        
        Returns:
            List of GPUInfo objects for NVIDIA GPUs
        """
        pass
    
    @abstractmethod
    def get_amd_gpus(self) -> List[GPUInfo]:
        """
        Get AMD GPU information.
        
        Returns:
            List of GPUInfo objects for AMD GPUs
        """
        pass
    
    @abstractmethod
    def get_intel_gpus(self) -> List[GPUInfo]:
        """
        Get Intel GPU information.
        
        Returns:
            List of GPUInfo objects for Intel GPUs
        """
        pass
    
    @abstractmethod
    def get_npu_info(self) -> Optional[NPUInfo]:
        """
        Get NPU information (AMD Ryzen AI, Intel VPU, etc).
        
        Returns:
            NPUInfo object if NPU detected, None otherwise
        """
        pass
    
    @abstractmethod
    def get_capabilities(self) -> HardwareCapabilities:
        """
        Detect hardware acceleration capabilities.
        
        Returns:
            HardwareCapabilities object with available backends
        """
        pass
    
    @staticmethod
    def get_os_version() -> str:
        """
        Get operating system version string.
        
        Returns:
            OS version string
        """
        try:
            return platform.platform()
        except Exception as e:
            logger.error(f"Failed to get OS version: {e}")
            return f"Unknown (error: {e})"
    
    @staticmethod
    def _classify_nvidia_gpu(gpu_name: str) -> GPUType:
        """
        Classify NVIDIA GPU as discrete or integrated.
        
        Most NVIDIA GPUs are discrete. This uses keyword matching
        for explicit classification.
        
        Args:
            gpu_name: GPU name from system
            
        Returns:
            GPUType enum value
        """
        name_lower = gpu_name.lower()
        
        # Check against discrete keywords
        for keyword in NVIDIAGPUKeyword:
            if keyword.value in name_lower:
                return GPUType.DISCRETE
        
        # Default to discrete for NVIDIA
        if GPUVendor.NVIDIA.value in name_lower:
            return GPUType.DISCRETE
        
        return GPUType.UNKNOWN
    
    @staticmethod
    def _classify_amd_gpu(gpu_name: str) -> GPUType:
        """
        Classify AMD GPU as discrete or integrated.
        
        Uses keyword matching. If no discrete keywords found,
        assumes integrated GPU.
        
        Args:
            gpu_name: GPU name from system
            
        Returns:
            GPUType enum value
        """
        name_lower = gpu_name.lower()
        
        # Check against discrete keywords
        for keyword in AMDGPUKeyword:
            if keyword.value in name_lower:
                return GPUType.DISCRETE
        
        # If no discrete keywords, assume integrated
        if GPUVendor.AMD.value in name_lower and "radeon" in name_lower:
            return GPUType.INTEGRATED
        
        return GPUType.UNKNOWN
    
    @staticmethod
    def _run_command(
        command: List[str],
        timeout: int,
        suppress_errors: bool = True
    ) -> Tuple[bool, Optional[str]]:
        """
        Run external command and capture output.
        
        Args:
            command: Command and arguments as list
            timeout: Timeout in seconds
            suppress_errors: If True, don't log errors
            
        Returns:
            Tuple of (success: bool, output: Optional[str])
        """
        try:
            result = subprocess.run(
                command,
                capture_output=True,
                text=True,
                timeout=timeout,
                check=True
            )
            return (True, result.stdout.strip())
        except subprocess.TimeoutExpired:
            if not suppress_errors:
                logger.warning(f"Command timed out: {' '.join(command)}")
            return (False, None)
        except subprocess.CalledProcessError as e:
            if not suppress_errors:
                logger.warning(f"Command failed: {' '.join(command)}: {e}")
            return (False, None)
        except FileNotFoundError:
            if not suppress_errors:
                logger.debug(f"Command not found: {command[0]}")
            return (False, None)
        except Exception as e:
            if not suppress_errors:
                logger.error(f"Command execution error: {' '.join(command)}: {e}")
            return (False, None)
    
    def _detect_nvidia_vram_smi(self) -> List[int]:
        """
        Detect NVIDIA GPU VRAM using nvidia-smi command.
        
        Returns:
            List of VRAM amounts in MB (one per GPU)
        """
        command = [
            "nvidia-smi",
            "--query-gpu=memory.total",
            "--format=csv,noheader,nounits"
        ]
        
        success, output = self._run_command(
            command,
            timeout=CommandTimeout.NVIDIA_SMI.value,
            suppress_errors=True
        )
        
        if not success or not output:
            return []
        
        vram_list: List[int] = []
        for line in output.split('\n'):
            line = line.strip()
            if line:
                try:
                    # nvidia-smi returns MB
                    vram_mb = int(line)
                    vram_list.append(vram_mb)
                except ValueError:
                    logger.warning(f"Could not parse VRAM value: {line}")
                    continue
        
        return vram_list
    
    def _detect_nvidia_driver_version_smi(self) -> Optional[str]:
        """
        Detect NVIDIA driver version using nvidia-smi.
        
        Returns:
            Driver version string or None
        """
        command = [
            "nvidia-smi",
            "--query-gpu=driver_version",
            "--format=csv,noheader,nounits"
        ]
        
        success, output = self._run_command(
            command,
            timeout=CommandTimeout.NVIDIA_SMI.value,
            suppress_errors=True
        )
        
        if success and output:
            # Return first line (in case of multiple GPUs, driver is same)
            version = output.split('\n')[0].strip()
            return version if version else None
        
        return None


class WindowsHardwareDetector(HardwareDetector):
    """
    Windows-specific hardware detection using WMI and system commands.
    """
    
    def __init__(self):
        """Initialize Windows hardware detector with WMI connection"""
        super().__init__()
        try:
            import wmi
            self.wmi_connection = wmi.WMI()
            logger.info("WMI connection established")
        except ImportError:
            logger.error("WMI module not available - install with: pip install wmi")
            raise RuntimeError("WMI module required for Windows hardware detection")
        except Exception as e:
            logger.error(f"Failed to initialize WMI: {e}")
            raise
    
    def get_cpu_info(self) -> CPUInfo:
        """
        Get CPU information using WMI.
        
        Returns:
            CPUInfo object with CPU details
        """
        try:
            processors = self.wmi_connection.Win32_Processor()
            if not processors:
                return CPUInfo(
                    name="Unknown CPU",
                    cores=0,
                    threads=0,
                    available=False,
                    error="No CPU information found"
                )
            
            processor = processors[0]
            return CPUInfo(
                name=processor.Name.strip(),
                cores=processor.NumberOfCores,
                threads=processor.NumberOfLogicalProcessors,
                max_clock_speed_mhz=processor.MaxClockSpeed,
                available=True
            )
        
        except Exception as e:
            logger.error(f"CPU detection failed: {e}")
            return CPUInfo(
                name="Unknown CPU",
                cores=0,
                threads=0,
                available=False,
                error=f"CPU detection failed: {str(e)}"
            )
    
    def get_nvidia_gpus(self) -> List[GPUInfo]:
        """
        Get NVIDIA GPUs using WMI and nvidia-smi.
        
        Returns:
            List of GPUInfo objects for NVIDIA GPUs
        """
        gpu_list: List[GPUInfo] = []
        
        try:
            video_controllers = self.wmi_connection.Win32_VideoController()
            
            # Get VRAM for all NVIDIA GPUs via nvidia-smi
            vram_list = self._detect_nvidia_vram_smi()
            driver_version = self._detect_nvidia_driver_version_smi()
            
            nvidia_gpu_index = 0
            
            for controller in video_controllers:
                if not controller.Name:
                    continue
                
                name = controller.Name
                if GPUVendor.NVIDIA.value.upper() not in name.upper():
                    continue
                
                gpu_type = self._classify_nvidia_gpu(name)
                
                # Only include discrete GPUs
                if gpu_type != GPUType.DISCRETE:
                    continue
                
                # Get VRAM for this GPU
                vram_mb = None
                if nvidia_gpu_index < len(vram_list):
                    vram_mb = vram_list[nvidia_gpu_index]
                
                gpu_info = GPUInfo(
                    name=name,
                    vendor=GPUVendor.NVIDIA,
                    gpu_type=gpu_type,
                    vram_mb=vram_mb,
                    driver_version=driver_version,
                    available=True
                )
                
                gpu_list.append(gpu_info)
                nvidia_gpu_index += 1
                
                logger.info(f"Detected NVIDIA GPU: {name} ({vram_mb} MB)" if vram_mb else f"Detected NVIDIA GPU: {name}")
        
        except Exception as e:
            logger.error(f"NVIDIA GPU detection failed: {e}")
            # Return empty list on error
        
        return gpu_list
    
    def get_amd_gpus(self) -> List[GPUInfo]:
        """
        Get AMD GPUs using WMI.
        
        Returns:
            List of GPUInfo objects for AMD GPUs
        """
        gpu_list: List[GPUInfo] = []
        
        try:
            video_controllers = self.wmi_connection.Win32_VideoController()
            
            for controller in video_controllers:
                if not controller.Name:
                    continue
                
                name = controller.Name
                if GPUVendor.AMD.value.upper() not in name.upper():
                    continue
                
                gpu_type = self._classify_amd_gpu(name)
                
                # Get VRAM from WMI (best-effort)
                vram_mb = None
                if hasattr(controller, 'AdapterRAM') and controller.AdapterRAM:
                    try:
                        vram_bytes = int(controller.AdapterRAM)
                        if vram_bytes > 0:
                            vram_mb = int(vram_bytes / BYTES_PER_MB)
                    except (ValueError, TypeError):
                        pass
                
                gpu_info = GPUInfo(
                    name=name,
                    vendor=GPUVendor.AMD,
                    gpu_type=gpu_type,
                    vram_mb=vram_mb,
                    driver_version=None,  # Could be added via WMI queries
                    available=True
                )
                
                gpu_list.append(gpu_info)
                logger.info(f"Detected AMD GPU: {name} ({gpu_type.value})")
        
        except Exception as e:
            logger.error(f"AMD GPU detection failed: {e}")
        
        return gpu_list
    
    def get_intel_gpus(self) -> List[GPUInfo]:
        """
        Get Intel GPUs using WMI.
        
        Returns:
            List of GPUInfo objects for Intel GPUs
        """
        gpu_list: List[GPUInfo] = []
        
        try:
            video_controllers = self.wmi_connection.Win32_VideoController()
            
            for controller in video_controllers:
                if not controller.Name:
                    continue
                
                name = controller.Name
                if GPUVendor.INTEL.value.upper() not in name.upper():
                    continue
                
                # Intel GPUs are typically integrated
                gpu_type = GPUType.INTEGRATED
                if "arc" in name.lower():
                    # Intel Arc are discrete GPUs
                    gpu_type = GPUType.DISCRETE
                
                # Get VRAM from WMI (best-effort)
                vram_mb = None
                if hasattr(controller, 'AdapterRAM') and controller.AdapterRAM:
                    try:
                        vram_bytes = int(controller.AdapterRAM)
                        if vram_bytes > 0:
                            vram_mb = int(vram_bytes / BYTES_PER_MB)
                    except (ValueError, TypeError):
                        pass
                
                gpu_info = GPUInfo(
                    name=name,
                    vendor=GPUVendor.INTEL,
                    gpu_type=gpu_type,
                    vram_mb=vram_mb,
                    driver_version=None,
                    available=True
                )
                
                gpu_list.append(gpu_info)
                logger.info(f"Detected Intel GPU: {name} ({gpu_type.value})")
        
        except Exception as e:
            logger.error(f"Intel GPU detection failed: {e}")
        
        return gpu_list
    
    def get_npu_info(self) -> Optional[NPUInfo]:
        """
        Get NPU information (AMD Ryzen AI, Intel VPU).
        
        Detects AMD Ryzen AI and Intel VPU NPUs with full capabilities.
        
        Returns:
            NPUInfo object if NPU detected, None otherwise
        """
        # Try AMD Ryzen AI NPU first
        amd_npu = self._detect_amd_npu()
        if amd_npu:
            return amd_npu
        
        # Try Intel VPU
        intel_vpu = self._detect_intel_vpu()
        if intel_vpu:
            return intel_vpu
        
        return None
    
    def _detect_amd_npu(self) -> Optional[NPUInfo]:
        """
        Detect AMD Ryzen AI NPU.
        
        Returns:
            NPUInfo if AMD NPU found, None otherwise
        """
        try:
            # Check for AMD NPU driver via WMI
            drivers = self.wmi_connection.Win32_PnPSignedDriver(
                DeviceName="NPU Compute Accelerator Device"
            )
            
            if not drivers:
                logger.debug("No AMD NPU driver found via WMI")
                return None
            
            driver = drivers[0]
            if not driver.DriverVersion:
                logger.debug("AMD NPU driver found but no version")
                return None
            
            logger.info(f"AMD NPU detected (driver: {driver.DriverVersion})")
            
            # Get NPU power mode via xrt-smi
            power_mode = self._get_npu_power_mode()
            
            return NPUInfo(
                name="AMD Ryzen AI NPU",
                driver_version=driver.DriverVersion,
                power_mode=power_mode,
                available=True
            )
            
        except Exception as e:
            logger.debug(f"AMD NPU detection failed: {e}")
            return None
    
    def _detect_intel_vpu(self) -> Optional[NPUInfo]:
        """
        Detect Intel VPU (Visual Processing Unit / NPU).
        
        Returns:
            NPUInfo if Intel VPU found, None otherwise
        """
        try:
            # Check for Intel VPU via WMI
            # Intel VPU shows up as display controller or compute accelerator
            video_controllers = self.wmi_connection.Win32_VideoController()
            
            for controller in video_controllers:
                if not controller.Name:
                    continue
                
                name = controller.Name.lower()
                
                # Check for Intel VPU keywords
                if "intel" in name and any(keyword in name for keyword in ["vpu", "npu", "meteor lake", "raptor lake"]):
                    logger.info(f"Intel VPU detected: {controller.Name}")
                    
                    # Get driver version
                    driver_version = None
                    if hasattr(controller, 'DriverVersion'):
                        driver_version = controller.DriverVersion
                    
                    return NPUInfo(
                        name=f"Intel VPU ({controller.Name})",
                        driver_version=driver_version,
                        available=True
                    )
            
            # Check for Intel NPU via processor name
            processors = self.wmi_connection.Win32_Processor()
            if processors:
                cpu_name = processors[0].Name.lower()
                # Intel Core Ultra (Meteor Lake+) has integrated NPU
                if "ultra" in cpu_name and "intel" in cpu_name:
                    logger.info(f"Intel NPU detected via CPU: {processors[0].Name}")
                    return NPUInfo(
                        name="Intel AI Boost (NPU)",
                        available=True
                    )
            
            logger.debug("No Intel VPU found")
            return None
            
        except Exception as e:
            logger.debug(f"Intel VPU detection failed: {e}")
            return None
    
    def _get_npu_power_mode(self) -> Optional[str]:
        """
        Get AMD NPU power mode via xrt-smi.exe.
        
        Returns:
            Power mode string (e.g., "turbo", "balanced") or None
        """
        try:
            # AMD NPU management tool path
            xrt_smi_path = r"C:\Windows\System32\AMD\xrt-smi.exe"
            
            result = subprocess.run(
                [xrt_smi_path, "examine", "-r", "platform"],
                capture_output=True,
                timeout=5,
                text=True,
                check=False
            )
            
            if result.returncode != 0:
                logger.debug("xrt-smi command failed or not found")
                return None
            
            # Parse power mode from output
            for line in result.stdout.splitlines():
                line = line.strip()
                if "Mode" in line:
                    # Extract mode value (last word in line)
                    parts = line.split()
                    if parts:
                        mode = parts[-1]
                        logger.debug(f"NPU power mode: {mode}")
                        return mode
            
            logger.debug("Could not parse power mode from xrt-smi output")
            return None
            
        except FileNotFoundError:
            logger.debug("xrt-smi.exe not found - NPU power mode unavailable")
            return None
        except subprocess.TimeoutExpired:
            logger.warning("xrt-smi command timed out")
            return None
        except Exception as e:
            logger.debug(f"Error getting NPU power mode: {e}")
            return None
    
    def get_capabilities(self) -> HardwareCapabilities:
        """
        Detect hardware acceleration capabilities.
        
        Returns:
            HardwareCapabilities object
        """
        capabilities = HardwareCapabilities()
        
        # Check CUDA
        try:
            import torch
            if torch.cuda.is_available():
                capabilities.has_cuda = True
                logger.info("CUDA is available")
        except ImportError:
            logger.debug("PyTorch not installed, cannot detect CUDA")
        except Exception as e:
            logger.debug(f"CUDA detection failed: {e}")
        
        # Check Vulkan
        success, _ = self._run_command(
            ["vulkaninfo", "--summary"],
            timeout=5,
            suppress_errors=True
        )
        if success:
            capabilities.has_vulkan = True
            logger.info("Vulkan is available")
        
        # Check DirectML (Windows only)
        try:
            import importlib.util
            if importlib.util.find_spec("onnxruntime"):
                import onnxruntime as ort
                if "DmlExecutionProvider" in ort.get_available_providers():
                    capabilities.has_directml = True
                    logger.info("DirectML is available")
        except Exception as e:
            logger.debug(f"DirectML detection failed: {e}")
        
        # Check NPU
        if self.get_npu_info() is not None:
            capabilities.has_npu = True
            logger.info("NPU is available")
        
        return capabilities


class LinuxHardwareDetector(HardwareDetector):
    """
    Linux-specific hardware detection using system commands and proc filesystem.
    """
    
    def __init__(self):
        """Initialize Linux hardware detector"""
        super().__init__()
        logger.info("Linux hardware detector initialized")
    
    def get_cpu_info(self) -> CPUInfo:
        """
        Get CPU information from /proc/cpuinfo.
        
        Returns:
            CPUInfo object with CPU details
        """
        try:
            with open('/proc/cpuinfo', 'r') as f:
                cpuinfo = f.read()
            
            # Parse CPU name
            cpu_name = "Unknown CPU"
            for line in cpuinfo.split('\n'):
                if line.startswith('model name'):
                    cpu_name = line.split(':')[1].strip()
                    break
            
            # Count physical cores and threads
            cores = 0
            threads = 0
            
            for line in cpuinfo.split('\n'):
                if line.startswith('cpu cores'):
                    cores = int(line.split(':')[1].strip())
                elif line.startswith('siblings'):
                    threads = int(line.split(':')[1].strip())
            
            return CPUInfo(
                name=cpu_name,
                cores=cores if cores > 0 else threads,
                threads=threads if threads > 0 else cores,
                available=True
            )
            
        except Exception as e:
            logger.error(f"CPU detection failed: {e}")
            return CPUInfo(
                name="Unknown CPU",
                cores=0,
                threads=0,
                available=False,
                error=str(e)
            )
    
    def get_nvidia_gpus(self) -> List[GPUInfo]:
        """
        Get NVIDIA GPUs using nvidia-smi.
        
        Returns:
            List of GPUInfo objects
        """
        gpu_list: List[GPUInfo] = []
        
        # Get GPU names and VRAM via nvidia-smi
        command = [
            "nvidia-smi",
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits"
        ]
        
        success, output = self._run_command(
            command,
            timeout=CommandTimeout.NVIDIA_SMI.value,
            suppress_errors=True
        )
        
        if not success or not output:
            return gpu_list
        
        # Parse output
        for line in output.split('\n'):
            line = line.strip()
            if not line:
                continue
            
            try:
                parts = line.split(',')
                if len(parts) >= 2:
                    name = parts[0].strip()
                    vram_mb = int(parts[1].strip())
                    
                    gpu_info = GPUInfo(
                        name=name,
                        vendor=GPUVendor.NVIDIA,
                        gpu_type=self._classify_nvidia_gpu(name),
                        vram_mb=vram_mb,
                        available=True
                    )
                    
                    gpu_list.append(gpu_info)
                    logger.info(f"Detected NVIDIA GPU: {name} ({vram_mb}MB)")
                    
            except (ValueError, IndexError) as e:
                logger.warning(f"Error parsing nvidia-smi output: {e}")
                continue
        
        return gpu_list
    
    def get_amd_gpus(self) -> List[GPUInfo]:
        """
        Get AMD GPUs using lspci and rocm-smi.
        
        Returns:
            List of GPUInfo objects
        """
        gpu_list: List[GPUInfo] = []
        
        # Try lspci for AMD GPU detection
        success, output = self._run_command(
            ["lspci"],
            timeout=5,
            suppress_errors=True
        )
        
        if not success or not output:
            return gpu_list
        
        # Parse lspci output for AMD GPUs
        for line in output.split('\n'):
            line_lower = line.lower()
            if 'vga' not in line_lower and 'display' not in line_lower:
                continue
            
            if 'amd' not in line_lower and 'ati' not in line_lower:
                continue
            
            # Extract GPU name
            parts = line.split(':')
            if len(parts) >= 3:
                name = ':'.join(parts[2:]).strip()
                
                gpu_info = GPUInfo(
                    name=name,
                    vendor=GPUVendor.AMD,
                    gpu_type=self._classify_amd_gpu(name),
                    available=True
                )
                
                gpu_list.append(gpu_info)
                logger.info(f"Detected AMD GPU: {name}")
        
        return gpu_list
    
    def get_intel_gpus(self) -> List[GPUInfo]:
        """
        Get Intel GPUs using lspci.
        
        Returns:
            List of GPUInfo objects
        """
        gpu_list: List[GPUInfo] = []
        
        success, output = self._run_command(
            ["lspci"],
            timeout=5,
            suppress_errors=True
        )
        
        if not success or not output:
            return gpu_list
        
        # Parse lspci output for Intel GPUs
        for line in output.split('\n'):
            line_lower = line.lower()
            if 'vga' not in line_lower and 'display' not in line_lower:
                continue
            
            if 'intel' not in line_lower:
                continue
            
            # Extract GPU name
            parts = line.split(':')
            if len(parts) >= 3:
                name = ':'.join(parts[2:]).strip()
                
                # Classify as discrete (Arc) or integrated
                gpu_type = GPUType.DISCRETE if 'arc' in name.lower() else GPUType.INTEGRATED
                
                gpu_info = GPUInfo(
                    name=name,
                    vendor=GPUVendor.INTEL,
                    gpu_type=gpu_type,
                    available=True
                )
                
                gpu_list.append(gpu_info)
                logger.info(f"Detected Intel GPU: {name}")
        
        return gpu_list
    
    def get_npu_info(self) -> Optional[NPUInfo]:
        """
        Get NPU information.
        
        Linux NPU support is limited currently.
        
        Returns:
            None (NPU detection on Linux TODO)
        """
        return None
    
    def get_capabilities(self) -> HardwareCapabilities:
        """
        Detect hardware acceleration capabilities on Linux.
        
        Returns:
            HardwareCapabilities object
        """
        capabilities = HardwareCapabilities()
        
        # Check CUDA
        try:
            import torch
            if torch.cuda.is_available():
                capabilities.has_cuda = True
                logger.info("CUDA is available")
        except ImportError:
            logger.debug("PyTorch not installed")
        except Exception as e:
            logger.debug(f"CUDA detection failed: {e}")
        
        # Check Vulkan
        success, _ = self._run_command(
            ["vulkaninfo", "--summary"],
            timeout=5,
            suppress_errors=True
        )
        if success:
            capabilities.has_vulkan = True
            logger.info("Vulkan is available")
        
        # Check ROCm
        success, _ = self._run_command(
            ["rocm-smi"],
            timeout=5,
            suppress_errors=True
        )
        if success:
            capabilities.has_rocm = True
            logger.info("ROCm is available")
        
        return capabilities


class MacOSHardwareDetector(HardwareDetector):
    """
    macOS-specific hardware detection using system_profiler and sysctl.
    """
    
    def __init__(self):
        """Initialize macOS hardware detector"""
        super().__init__()
        logger.info("macOS hardware detector initialized")
    
    def get_cpu_info(self) -> CPUInfo:
        """
        Get CPU information using sysctl.
        
        Returns:
            CPUInfo object with CPU details
        """
        try:
            # Get CPU name
            success, cpu_name = self._run_command(
                ["sysctl", "-n", "machdep.cpu.brand_string"],
                timeout=5,
                suppress_errors=False
            )
            
            if not success or not cpu_name:
                cpu_name = "Unknown CPU"
            
            # Get cores
            success, cores_str = self._run_command(
                ["sysctl", "-n", "hw.physicalcpu"],
                timeout=5,
                suppress_errors=True
            )
            cores = int(cores_str) if success and cores_str else 0
            
            # Get threads
            success, threads_str = self._run_command(
                ["sysctl", "-n", "hw.logicalcpu"],
                timeout=5,
                suppress_errors=True
            )
            threads = int(threads_str) if success and threads_str else cores
            
            # Get clock speed
            success, freq_str = self._run_command(
                ["sysctl", "-n", "hw.cpufrequency_max"],
                timeout=5,
                suppress_errors=True
            )
            max_clock_mhz = int(int(freq_str) / 1000000) if success and freq_str else None
            
            return CPUInfo(
                name=cpu_name.strip(),
                cores=cores,
                threads=threads,
                max_clock_speed_mhz=max_clock_mhz,
                available=True
            )
            
        except Exception as e:
            logger.error(f"CPU detection failed: {e}")
            return CPUInfo(
                name="Unknown CPU",
                cores=0,
                threads=0,
                available=False,
                error=str(e)
            )
    
    def get_nvidia_gpus(self) -> List[GPUInfo]:
        """
        Get NVIDIA GPUs using system_profiler.
        
        Returns:
            List of GPUInfo objects
        """
        gpu_list: List[GPUInfo] = []
        
        success, output = self._run_command(
            ["system_profiler", "SPDisplaysDataType"],
            timeout=10,
            suppress_errors=True
        )
        
        if not success or not output:
            return gpu_list
        
        # Parse system_profiler output for NVIDIA
        lines = output.split('\n')
        for i, line in enumerate(lines):
            if 'NVIDIA' in line or 'GeForce' in line or 'Quadro' in line:
                name = line.split(':')[0].strip() if ':' in line else line.strip()
                
                # Try to find VRAM
                vram_mb = None
                for j in range(i, min(i+10, len(lines))):
                    if 'VRAM' in lines[j] or 'Memory' in lines[j]:
                        vram_line = lines[j]
                        # Extract VRAM amount
                        if 'GB' in vram_line:
                            try:
                                vram_gb = float(vram_line.split('GB')[0].split()[-1])
                                vram_mb = int(vram_gb * 1024)
                            except (ValueError, IndexError):
                                pass
                        break
                
                gpu_info = GPUInfo(
                    name=name,
                    vendor=GPUVendor.NVIDIA,
                    gpu_type=self._classify_nvidia_gpu(name),
                    vram_mb=vram_mb,
                    available=True
                )
                
                gpu_list.append(gpu_info)
                logger.info(f"Detected NVIDIA GPU: {name}")
        
        return gpu_list
    
    def get_amd_gpus(self) -> List[GPUInfo]:
        """
        Get AMD GPUs using system_profiler.
        
        Returns:
            List of GPUInfo objects (typically empty on macOS)
        """
        # AMD GPUs are rare on macOS
        return []
    
    def get_intel_gpus(self) -> List[GPUInfo]:
        """
        Get Intel GPUs using system_profiler.
        
        Returns:
            List of GPUInfo objects
        """
        gpu_list: List[GPUInfo] = []
        
        success, output = self._run_command(
            ["system_profiler", "SPDisplaysDataType"],
            timeout=10,
            suppress_errors=True
        )
        
        if not success or not output:
            return gpu_list
        
        # Parse for Intel GPUs
        lines = output.split('\n')
        for line in lines:
            if 'Intel' in line and ('Graphics' in line or 'UHD' in line or 'Iris' in line):
                name = line.split(':')[0].strip() if ':' in line else line.strip()
                
                gpu_info = GPUInfo(
                    name=name,
                    vendor=GPUVendor.INTEL,
                    gpu_type=GPUType.INTEGRATED,  # Intel GPUs on Mac are integrated
                    available=True
                )
                
                gpu_list.append(gpu_info)
                logger.info(f"Detected Intel GPU: {name}")
        
        return gpu_list
    
    def get_npu_info(self) -> Optional[NPUInfo]:
        """
        Get NPU information.
        
        macOS doesn't have separate NPU (Neural Engine is part of Apple Silicon).
        
        Returns:
            None
        """
        return None
    
    def get_capabilities(self) -> HardwareCapabilities:
        """
        Detect hardware acceleration capabilities on macOS.
        
        Returns:
            HardwareCapabilities object
        """
        capabilities = HardwareCapabilities()
        
        # Check Metal (should be available on all modern Macs)
        success, _ = self._run_command(
            ["system_profiler", "SPDisplaysDataType"],
            timeout=10,
            suppress_errors=True
        )
        if success:
            capabilities.has_metal = True
            logger.info("Metal is available")
        
        # Check CUDA (rare on modern Macs)
        try:
            import torch
            if torch.cuda.is_available():
                capabilities.has_cuda = True
                logger.info("CUDA is available")
        except ImportError:
            logger.debug("PyTorch not installed")
        except Exception:
            pass
        
        # Check Vulkan
        success, _ = self._run_command(
            ["vulkaninfo", "--summary"],
            timeout=5,
            suppress_errors=True
        )
        if success:
            capabilities.has_vulkan = True
            logger.info("Vulkan is available")
        
        return capabilities


def create_hardware_detector() -> HardwareDetector:
    """
    Factory function to create OS-specific hardware detector.
    
    Returns:
        HardwareDetector instance for current OS
        
    Raises:
        NotImplementedError: If OS is not supported
    """
    os_type = HardwareDetector._detect_os()
    
    if os_type == OSType.WINDOWS:
        return WindowsHardwareDetector()
    elif os_type == OSType.LINUX:
        return LinuxHardwareDetector()
    elif os_type == OSType.DARWIN:
        return MacOSHardwareDetector()
    else:
        raise NotImplementedError(f"Unsupported OS: {os_type.value}")

