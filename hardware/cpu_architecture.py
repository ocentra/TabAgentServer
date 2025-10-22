"""
CPU Microarchitecture Detection for BitNet Binary Selection.

Detects specific CPU microarchitectures (Zen2, Zen3, Alderlake, etc.) to select
the optimal BitNet/llama.cpp binary variant from BitnetRelease/ folder.

This is critical because BitNet provides optimized binaries for specific CPU 
microarchitectures that can be 2-5x faster than portable builds.
"""

import platform
import subprocess
import logging
import re
from typing import Optional, Dict, Any
from enum import Enum
from dataclasses import dataclass
from pathlib import Path


logger = logging.getLogger(__name__)


class CPUVendor(str, Enum):
    """CPU vendors"""
    INTEL = "intel"
    AMD = "amd"
    APPLE = "apple"
    ARM = "arm"
    UNKNOWN = "unknown"


class CPUArchitecture(str, Enum):
    """
    CPU Microarchitectures supported by BitNet/llama.cpp.
    
    Maps to BitnetRelease variant folders:
    - bitnet-portable: Universal fallback (slowest)
    - bitnet-amd-zen1: AMD Ryzen 1000 series
    - bitnet-amd-zen2: AMD Ryzen 3000 series  
    - bitnet-amd-zen3: AMD Ryzen 5000 series
    - bitnet-amd-zen4: AMD Ryzen 7000 series
    - bitnet-amd-zen5: AMD Ryzen 9000 series
    - bitnet-intel-haswell: Intel 4th-5th gen (2013-2015)
    - bitnet-intel-broadwell: Intel 5th gen mobile (2014-2015)
    - bitnet-intel-skylake: Intel 6th-11th gen (2015-2021)
    - bitnet-intel-icelake: Intel 10th-11th gen (2019-2021)
    - bitnet-intel-rocketlake: Intel 11th gen desktop (2021)
    - bitnet-intel-alderlake: Intel 12th+ gen (2021+)
    """
    # AMD Microarchitectures
    AMD_ZEN1 = "bitnet-amd-zen1"
    AMD_ZEN2 = "bitnet-amd-zen2"
    AMD_ZEN3 = "bitnet-amd-zen3"
    AMD_ZEN4 = "bitnet-amd-zen4"
    AMD_ZEN5 = "bitnet-amd-zen5"
    
    # Intel Microarchitectures
    INTEL_HASWELL = "bitnet-intel-haswell"
    INTEL_BROADWELL = "bitnet-intel-broadwell"
    INTEL_SKYLAKE = "bitnet-intel-skylake"
    INTEL_ICELAKE = "bitnet-intel-icelake"
    INTEL_ROCKETLAKE = "bitnet-intel-rocketlake"
    INTEL_ALDERLAKE = "bitnet-intel-alderlake"
    
    # Fallback
    PORTABLE = "bitnet-portable"
    STANDARD = "standard"  # Generic llama.cpp (not BitNet)


@dataclass(frozen=True)
class CPUArchitectureInfo:
    """
    Detailed CPU architecture information.
    
    Attributes:
        vendor: CPU vendor (Intel, AMD, etc.)
        architecture: Detected microarchitecture
        model_name: Full CPU model name
        family: CPU family number
        model: CPU model number
        stepping: CPU stepping
        features: CPU feature flags
        variant_path: Path to optimal BitNet binary variant
    """
    vendor: CPUVendor
    architecture: CPUArchitecture
    model_name: str
    family: Optional[int] = None
    model: Optional[int] = None
    stepping: Optional[int] = None
    features: Optional[set] = None
    variant_path: Optional[str] = None


class CPUArchitectureDetector:
    """
    Detects CPU microarchitecture for optimal binary selection.
    
    Uses multiple detection methods:
    1. CPUID (via wmic/lscpu/sysctl)
    2. Model name pattern matching
    3. CPU feature flags
    """
    
    def __init__(self):
        """Initialize CPU architecture detector"""
        self.os_type = platform.system()
        logger.info(f"CPUArchitectureDetector initialized for {self.os_type}")
    
    def detect(self) -> CPUArchitectureInfo:
        """
        Detect CPU microarchitecture.
        
        Returns:
            CPUArchitectureInfo with detected architecture
        """
        if self.os_type == "Windows":
            return self._detect_windows()
        elif self.os_type == "Linux":
            return self._detect_linux()
        elif self.os_type == "Darwin":
            return self._detect_macos()
        else:
            logger.warning(f"Unsupported OS: {self.os_type}, using portable variant")
            return CPUArchitectureInfo(
                vendor=CPUVendor.UNKNOWN,
                architecture=CPUArchitecture.PORTABLE,
                model_name="Unknown CPU"
            )
    
    def _detect_windows(self) -> CPUArchitectureInfo:
        """Detect CPU architecture on Windows using PowerShell"""
        try:
            # Get CPU info via PowerShell (modern method, wmic is deprecated)
            ps_command = "Get-CimInstance -ClassName Win32_Processor | Select-Object Name, Manufacturer, NumberOfCores | ConvertTo-Json"
            
            result = subprocess.run(
                ["powershell", "-Command", ps_command],
                capture_output=True,
                text=True,
                timeout=5,
                check=True
            )
            
            import json
            cpu_data = json.loads(result.stdout)
            
            # Handle single CPU (dict) or multiple CPUs (list)
            if isinstance(cpu_data, list):
                cpu_data = cpu_data[0]
            
            model_name = cpu_data.get('Name', 'Unknown CPU').strip()
            manufacturer = cpu_data.get('Manufacturer', '').strip().lower()
            
            # Detect vendor
            if 'intel' in manufacturer or 'intel' in model_name.lower():
                vendor = CPUVendor.INTEL
            elif 'amd' in manufacturer or 'amd' in model_name.lower():
                vendor = CPUVendor.AMD
            else:
                vendor = CPUVendor.UNKNOWN
            
            # Detect architecture based on model name
            architecture = self._detect_architecture_from_model(model_name, vendor)
            
            # Get CPUID info for more accurate detection
            family, model, stepping = self._get_cpuid_windows()
            
            # Refine architecture based on CPUID if available
            if family is not None and model is not None:
                architecture = self._refine_architecture(
                    architecture, vendor, family, model, stepping
                )
            
            return CPUArchitectureInfo(
                vendor=vendor,
                architecture=architecture,
                model_name=model_name,
                family=family,
                model=model,
                stepping=stepping
            )
            
        except Exception as e:
            logger.error(f"Windows CPU detection failed: {e}")
            return CPUArchitectureInfo(
                vendor=CPUVendor.UNKNOWN,
                architecture=CPUArchitecture.PORTABLE,
                model_name="Unknown CPU"
            )
    
    def _detect_linux(self) -> CPUArchitectureInfo:
        """Detect CPU architecture on Linux using /proc/cpuinfo"""
        try:
            with open('/proc/cpuinfo', 'r') as f:
                cpuinfo = f.read()
            
            # Parse cpuinfo
            info = {}
            for line in cpuinfo.split('\n'):
                if ':' in line:
                    key, value = line.split(':', 1)
                    key = key.strip()
                    value = value.strip()
                    if key not in info:  # Take first occurrence
                        info[key] = value
            
            model_name = info.get('model name', 'Unknown CPU')
            vendor_id = info.get('vendor_id', '').lower()
            
            # Detect vendor
            if 'intel' in vendor_id or 'genuineintel' in vendor_id:
                vendor = CPUVendor.INTEL
            elif 'amd' in vendor_id or 'authenticamd' in vendor_id:
                vendor = CPUVendor.AMD
            else:
                vendor = CPUVendor.UNKNOWN
            
            # Get CPUID info
            family = int(info.get('cpu family', 0))
            model = int(info.get('model', 0))
            stepping = int(info.get('stepping', 0))
            
            # Get CPU flags
            flags = set(info.get('flags', '').split())
            
            # Detect architecture
            architecture = self._detect_architecture_from_model(model_name, vendor)
            architecture = self._refine_architecture(
                architecture, vendor, family, model, stepping, flags
            )
            
            return CPUArchitectureInfo(
                vendor=vendor,
                architecture=architecture,
                model_name=model_name,
                family=family,
                model=model,
                stepping=stepping,
                features=flags
            )
            
        except Exception as e:
            logger.error(f"Linux CPU detection failed: {e}")
            return CPUArchitectureInfo(
                vendor=CPUVendor.UNKNOWN,
                architecture=CPUArchitecture.PORTABLE,
                model_name="Unknown CPU"
            )
    
    def _detect_macos(self) -> CPUArchitectureInfo:
        """Detect CPU architecture on macOS using sysctl"""
        try:
            # Get CPU name
            result = subprocess.run(
                ["sysctl", "-n", "machdep.cpu.brand_string"],
                capture_output=True,
                text=True,
                timeout=5,
                check=True
            )
            model_name = result.stdout.strip()
            
            # Detect if Apple Silicon
            if 'apple' in model_name.lower() or platform.machine() == 'arm64':
                # Apple Silicon Macs - use portable for now
                # (Future: could have ARM-specific builds)
                return CPUArchitectureInfo(
                    vendor=CPUVendor.APPLE,
                    architecture=CPUArchitecture.PORTABLE,
                    model_name=model_name
                )
            
            # Intel Mac
            vendor = CPUVendor.INTEL
            architecture = self._detect_architecture_from_model(model_name, vendor)
            
            # Try to get more detailed info
            try:
                result = subprocess.run(
                    ["sysctl", "-n", "machdep.cpu.family"],
                    capture_output=True,
                    text=True,
                    timeout=5,
                    check=True
                )
                family = int(result.stdout.strip())
                
                result = subprocess.run(
                    ["sysctl", "-n", "machdep.cpu.model"],
                    capture_output=True,
                    text=True,
                    timeout=5,
                    check=True
                )
                model = int(result.stdout.strip())
                
                architecture = self._refine_architecture(
                    architecture, vendor, family, model, None
                )
                
                return CPUArchitectureInfo(
                    vendor=vendor,
                    architecture=architecture,
                    model_name=model_name,
                    family=family,
                    model=model
                )
                
            except Exception:
                # Fallback to name-based detection
                return CPUArchitectureInfo(
                    vendor=vendor,
                    architecture=architecture,
                    model_name=model_name
                )
            
        except Exception as e:
            logger.error(f"macOS CPU detection failed: {e}")
            return CPUArchitectureInfo(
                vendor=CPUVendor.UNKNOWN,
                architecture=CPUArchitecture.PORTABLE,
                model_name="Unknown CPU"
            )
    
    def _detect_architecture_from_model(
        self,
        model_name: str,
        vendor: CPUVendor
    ) -> CPUArchitecture:
        """
        Detect microarchitecture from CPU model name.
        
        This is a coarse detection - should be refined with CPUID.
        """
        model_lower = model_name.lower()
        
        if vendor == CPUVendor.AMD:
            # AMD Ryzen detection
            if 'ryzen' in model_lower:
                # Ryzen 9000 series (Zen 5)
                if any(x in model_lower for x in ['9950', '9900', '9700', '9600']):
                    return CPUArchitecture.AMD_ZEN5
                # Ryzen 7000 series (Zen 4)
                elif any(x in model_lower for x in ['7950', '7900', '7700', '7600']):
                    return CPUArchitecture.AMD_ZEN4
                # Ryzen 5000 series (Zen 3)
                elif any(x in model_lower for x in ['5950', '5900', '5800', '5700', '5600']):
                    return CPUArchitecture.AMD_ZEN3
                # Ryzen 3000 series (Zen 2)
                elif any(x in model_lower for x in ['3950', '3900', '3700', '3600', '3300']):
                    return CPUArchitecture.AMD_ZEN2
                # Ryzen 2000 series (Zen+) - treat as Zen2
                elif any(x in model_lower for x in ['2700', '2600', '2400', '2200']):
                    return CPUArchitecture.AMD_ZEN2
                # Ryzen 1000 series (Zen 1)
                elif any(x in model_lower for x in ['1800', '1700', '1600', '1500', '1400']):
                    return CPUArchitecture.AMD_ZEN1
            
            # EPYC detection
            if 'epyc' in model_lower:
                if '9' in model_lower:  # EPYC 9004 series (Zen 4)
                    return CPUArchitecture.AMD_ZEN4
                elif '7' in model_lower:  # EPYC 7003 series (Zen 3)
                    return CPUArchitecture.AMD_ZEN3
                else:  # Older EPYC
                    return CPUArchitecture.AMD_ZEN2
        
        elif vendor == CPUVendor.INTEL:
            # Intel Core generation detection
            
            # 12th gen+ (Alder Lake, Raptor Lake, Meteor Lake)
            if any(x in model_lower for x in ['12', '13', '14']) and 'gen' in model_lower:
                return CPUArchitecture.INTEL_ALDERLAKE
            if any(x in model_lower for x in ['12th', '13th', '14th']):
                return CPUArchitecture.INTEL_ALDERLAKE
            if any(x in model_lower for x in ['i9-12', 'i7-12', 'i5-12', 'i3-12']):
                return CPUArchitecture.INTEL_ALDERLAKE
            if any(x in model_lower for x in ['i9-13', 'i7-13', 'i5-13', 'i3-13']):
                return CPUArchitecture.INTEL_ALDERLAKE
            if any(x in model_lower for x in ['i9-14', 'i7-14', 'i5-14', 'i3-14']):
                return CPUArchitecture.INTEL_ALDERLAKE
            
            # 11th gen (Rocket Lake desktop, Ice Lake mobile)
            if 'rocket lake' in model_lower:
                return CPUArchitecture.INTEL_ROCKETLAKE
            if any(x in model_lower for x in ['i9-11', 'i7-11', 'i5-11', 'i3-11']):
                if 'k' in model_lower or 'desktop' in model_lower:
                    return CPUArchitecture.INTEL_ROCKETLAKE
                else:
                    return CPUArchitecture.INTEL_ICELAKE
            
            # 10th gen (Ice Lake, Comet Lake)
            if 'ice lake' in model_lower:
                return CPUArchitecture.INTEL_ICELAKE
            if any(x in model_lower for x in ['i9-10', 'i7-10', 'i5-10', 'i3-10']):
                # Ice Lake has -G suffix, Comet Lake doesn't
                if '-g' in model_lower or 'ice' in model_lower:
                    return CPUArchitecture.INTEL_ICELAKE
                else:
                    return CPUArchitecture.INTEL_SKYLAKE  # Comet Lake is Skylake refresh
            
            # 6th-9th gen (Skylake and derivatives)
            if any(x in model_lower for x in ['6th', '7th', '8th', '9th']):
                return CPUArchitecture.INTEL_SKYLAKE
            if any(x in model_lower for x in ['i9-9', 'i7-9', 'i5-9', 'i3-9',
                                               'i7-8', 'i5-8', 'i3-8',
                                               'i7-7', 'i5-7', 'i3-7',
                                               'i7-6', 'i5-6', 'i3-6']):
                return CPUArchitecture.INTEL_SKYLAKE
            
            # 5th gen (Broadwell)
            if 'broadwell' in model_lower or '5th' in model_lower:
                return CPUArchitecture.INTEL_BROADWELL
            if any(x in model_lower for x in ['i7-5', 'i5-5', 'i3-5']):
                return CPUArchitecture.INTEL_BROADWELL
            
            # 4th gen (Haswell)
            if 'haswell' in model_lower or '4th' in model_lower:
                return CPUArchitecture.INTEL_HASWELL
            if any(x in model_lower for x in ['i7-4', 'i5-4', 'i3-4']):
                return CPUArchitecture.INTEL_HASWELL
            
            # Xeon detection (rough)
            if 'xeon' in model_lower:
                # Modern Xeons
                if any(x in model_lower for x in ['platinum', 'gold', 'silver']):
                    return CPUArchitecture.INTEL_SKYLAKE  # Skylake-SP or newer
                else:
                    return CPUArchitecture.INTEL_HASWELL  # Older Xeons
        
        # Fallback to portable
        logger.warning(f"Could not detect specific architecture for {model_name}, using portable")
        return CPUArchitecture.PORTABLE
    
    def _refine_architecture(
        self,
        initial: CPUArchitecture,
        vendor: CPUVendor,
        family: Optional[int],
        model: Optional[int],
        stepping: Optional[int],
        features: Optional[set] = None
    ) -> CPUArchitecture:
        """
        Refine architecture detection using CPUID family/model numbers.
        
        This provides more accurate detection than model name alone.
        """
        if family is None or model is None:
            return initial
        
        if vendor == CPUVendor.AMD:
            # AMD Family 25 (0x19) = Zen 3/4
            if family == 25:
                # Model range for Zen 4
                if model >= 0x60:  # Raphael (Zen 4)
                    return CPUArchitecture.AMD_ZEN4
                else:  # Vermeer, Cezanne (Zen 3)
                    return CPUArchitecture.AMD_ZEN3
            
            # AMD Family 23 (0x17) = Zen 1/2/+
            elif family == 23:
                if model >= 0x30:  # Matisse (Zen 2)
                    return CPUArchitecture.AMD_ZEN2
                elif model >= 0x10:  # Pinnacle Ridge (Zen+)
                    return CPUArchitecture.AMD_ZEN2  # Treat Zen+ as Zen2
                else:  # Summit Ridge (Zen 1)
                    return CPUArchitecture.AMD_ZEN1
            
            # AMD Family 26 (0x1A) = Zen 5
            elif family == 26:
                return CPUArchitecture.AMD_ZEN5
        
        elif vendor == CPUVendor.INTEL:
            # Intel Family 6 (modern Intel CPUs)
            if family == 6:
                # Alder Lake and newer (12th gen+)
                if model in [0x97, 0x9A, 0xB7, 0xBA, 0xBF]:  # ADL, RPL, MTL
                    return CPUArchitecture.INTEL_ALDERLAKE
                
                # Rocket Lake (11th gen desktop)
                elif model in [0xA7]:
                    return CPUArchitecture.INTEL_ROCKETLAKE
                
                # Ice Lake (10th/11th gen mobile)
                elif model in [0x7D, 0x7E, 0x6A, 0x6C]:
                    return CPUArchitecture.INTEL_ICELAKE
                
                # Skylake and derivatives (6th-9th gen)
                elif model in [0x4E, 0x5E, 0x8E, 0x9E, 0xA5, 0xA6]:
                    return CPUArchitecture.INTEL_SKYLAKE
                
                # Broadwell (5th gen)
                elif model in [0x3D, 0x47, 0x4F, 0x56]:
                    return CPUArchitecture.INTEL_BROADWELL
                
                # Haswell (4th gen)
                elif model in [0x3C, 0x3F, 0x45, 0x46]:
                    return CPUArchitecture.INTEL_HASWELL
        
        return initial
    
    def _get_cpuid_windows(self) -> tuple[Optional[int], Optional[int], Optional[int]]:
        """Get CPUID family, model, stepping on Windows"""
        try:
            # Use PowerShell to get CPUID info (use Get-CimInstance, modern replacement for Get-WmiObject)
            ps_command = "Get-CimInstance -ClassName Win32_Processor | Select-Object Level, Revision | ConvertTo-Json"
            
            result = subprocess.run(
                ["powershell", "-Command", ps_command],
                capture_output=True,
                text=True,
                timeout=5,
                check=True
            )
            
            import json
            cpu_data = json.loads(result.stdout)
            
            # Handle single CPU (dict) or multiple CPUs (list)
            if isinstance(cpu_data, list):
                cpu_data = cpu_data[0]
            
            family = cpu_data.get('Level')
            revision = cpu_data.get('Revision')
            
            if family is not None and revision is not None:
                # Decode revision into model and stepping
                # Revision format varies, but typically: model = high byte, stepping = low byte
                model = (revision >> 8) & 0xFF
                stepping = revision & 0xFF
                
                return (family, model, stepping)
                
        except Exception as e:
            logger.debug(f"Could not get CPUID via PowerShell: {e}")
        
        return (None, None, None)
    
    def get_optimal_variant_path(
        self,
        base_path: Path,
        compute_type: str = "cpu"
    ) -> Path:
        """
        Get path to optimal BitNet binary variant.
        
        Args:
            base_path: Path to BitnetRelease/ directory
            compute_type: "cpu" or "gpu"
            
        Returns:
            Path to optimal variant folder
        """
        arch_info = self.detect()
        
        # Build path: BitnetRelease/{compute_type}/{platform}/{variant}/
        system = platform.system().lower()
        variant = arch_info.architecture.value
        
        variant_path = base_path / compute_type / system / variant
        
        # Verify path exists
        if not variant_path.exists():
            logger.warning(
                f"Optimal variant path not found: {variant_path}, "
                f"falling back to portable"
            )
            variant_path = base_path / compute_type / system / "bitnet-portable"
            
            if not variant_path.exists():
                logger.error(f"Portable variant also not found at {variant_path}")
                # Try standard as last resort
                variant_path = base_path / compute_type / system / "standard"
        
        logger.info(f"Selected variant: {variant_path}")
        return variant_path


def get_optimal_binary_path(
    bitnet_release_dir: Path,
    binary_name: str = "llama.dll",
    compute_type: str = "cpu"
) -> Optional[Path]:
    """
    Get path to optimal BitNet binary for current system.
    
    Args:
        bitnet_release_dir: Path to BitnetRelease/ directory
        binary_name: Name of binary to find (llama.dll, llama-server.exe, etc.)
        compute_type: "cpu" or "gpu"
        
    Returns:
        Path to binary or None if not found
    """
    detector = CPUArchitectureDetector()
    variant_dir = detector.get_optimal_variant_path(bitnet_release_dir, compute_type)
    
    binary_path = variant_dir / binary_name
    
    if not binary_path.exists():
        logger.error(f"Binary not found: {binary_path}")
        return None
    
    return binary_path


# Convenience function for quick detection
def detect_cpu_architecture() -> CPUArchitectureInfo:
    """
    Quick CPU architecture detection.
    
    Returns:
        CPUArchitectureInfo with detected architecture
    """
    detector = CPUArchitectureDetector()
    return detector.detect()


if __name__ == "__main__":
    # Test the detector
    logging.basicConfig(level=logging.INFO)
    
    print("Detecting CPU architecture...")
    info = detect_cpu_architecture()
    
    print(f"\nCPU Information:")
    print(f"  Model: {info.model_name}")
    print(f"  Vendor: {info.vendor.value}")
    print(f"  Architecture: {info.architecture.value}")
    
    if info.family is not None:
        print(f"  Family: {info.family} (0x{info.family:X})")
    if info.model is not None:
        print(f"  Model: {info.model} (0x{info.model:X})")
    if info.stepping is not None:
        print(f"  Stepping: {info.stepping}")
    
    # Test path resolution
    base_path = Path(__file__).parent.parent / "BitNet" / "BitnetRelease"
    if base_path.exists():
        detector = CPUArchitectureDetector()
        variant_path = detector.get_optimal_variant_path(base_path, "cpu")
        print(f"\nOptimal variant path: {variant_path}")
        
        # Check if llama.dll exists
        dll_path = variant_path / ("llama.dll" if platform.system() == "Windows" else "libllama.so")
        if dll_path.exists():
            print(f"✓ Binary found: {dll_path}")
        else:
            print(f"✗ Binary not found: {dll_path}")

