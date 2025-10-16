"""
Command-line interface for TabAgent native host.

Provides testing and administration commands:
- System information
- Hardware detection
- Backend availability
- Model management
- Server control
"""

import sys
import argparse
import logging
import json
from typing import Optional, List
from enum import Enum

from core import (
    # Hardware detection
    create_hardware_detector,
    HardwareDetector,
    
    # Backend selection
    BackendSelector,
    AccelerationDetector,
    
    # Port management
    get_port_manager,
    
    # Types
    ModelType,
    ServerType,
    AccelerationBackend,
)


class ExitCode(int, Enum):
    """CLI exit codes"""
    SUCCESS = 0
    ERROR = 1
    INVALID_ARGS = 2
    NOT_IMPLEMENTED = 3


class OutputFormat(str, Enum):
    """Output format options"""
    TEXT = "text"
    JSON = "json"


def setup_logging(verbose: int) -> None:
    """
    Setup logging based on verbosity level.
    
    Args:
        verbose: Verbosity count (0=WARNING, 1=INFO, 2+=DEBUG)
    """
    if verbose == 0:
        level = logging.WARNING
    elif verbose == 1:
        level = logging.INFO
    else:
        level = logging.DEBUG
    
    logging.basicConfig(
        level=level,
        format='%(levelname)s: %(message)s'
    )


def cmd_info(args) -> int:
    """
    Display system information.
    
    Args:
        args: Parsed command arguments
        
    Returns:
        Exit code
    """
    try:
        detector = create_hardware_detector()
        hw_info = detector.get_hardware_info()
        
        if args.format == OutputFormat.JSON.value:
            # JSON output
            output = {
                "os": hw_info.os_version,
                "cpu": {
                    "name": hw_info.cpu.name,
                    "cores": hw_info.cpu.cores,
                    "threads": hw_info.cpu.threads,
                },
                "nvidia_gpus": [
                    {
                        "name": gpu.name,
                        "vram_mb": gpu.vram_mb,
                        "driver_version": gpu.driver_version,
                    }
                    for gpu in hw_info.nvidia_gpus
                ],
                "amd_gpus": [
                    {
                        "name": gpu.name,
                        "vram_mb": gpu.vram_mb,
                    }
                    for gpu in hw_info.amd_gpus
                ],
                "capabilities": {
                    "cuda": hw_info.capabilities.has_cuda,
                    "vulkan": hw_info.capabilities.has_vulkan,
                    "rocm": hw_info.capabilities.has_rocm,
                    "metal": hw_info.capabilities.has_metal,
                    "directml": hw_info.capabilities.has_directml,
                }
            }
            print(json.dumps(output, indent=2))
        else:
            # Text output
            print("=== TabAgent System Information ===\n")
            print(f"OS: {hw_info.os_version}\n")
            
            print("CPU:")
            print(f"  Name: {hw_info.cpu.name}")
            print(f"  Cores: {hw_info.cpu.cores}")
            print(f"  Threads: {hw_info.cpu.threads}")
            if hw_info.cpu.max_clock_speed_mhz:
                print(f"  Max Speed: {hw_info.cpu.max_clock_speed_mhz} MHz")
            print()
            
            if hw_info.nvidia_gpus:
                print("NVIDIA GPUs:")
                for gpu in hw_info.nvidia_gpus:
                    vram_str = f"{gpu.vram_mb} MB" if gpu.vram_mb else "Unknown"
                    driver_str = f" (Driver: {gpu.driver_version})" if gpu.driver_version else ""
                    print(f"  - {gpu.name} ({vram_str}){driver_str}")
                print()
            
            if hw_info.amd_gpus:
                print("AMD GPUs:")
                for gpu in hw_info.amd_gpus:
                    vram_str = f"{gpu.vram_mb} MB" if gpu.vram_mb else "Unknown"
                    print(f"  - {gpu.name} ({vram_str})")
                print()
            
            if hw_info.intel_gpus:
                print("Intel GPUs:")
                for gpu in hw_info.intel_gpus:
                    print(f"  - {gpu.name}")
                print()
            
            print("Hardware Acceleration:")
            capabilities = [
                ("CUDA", hw_info.capabilities.has_cuda),
                ("Vulkan", hw_info.capabilities.has_vulkan),
                ("ROCm", hw_info.capabilities.has_rocm),
                ("Metal", hw_info.capabilities.has_metal),
                ("DirectML", hw_info.capabilities.has_directml),
            ]
            for name, available in capabilities:
                status = "✓ Available" if available else "✗ Not available"
                print(f"  {name}: {status}")
        
        return ExitCode.SUCCESS.value
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return ExitCode.ERROR.value


def cmd_backends(args) -> int:
    """
    List available backends.
    
    Args:
        args: Parsed command arguments
        
    Returns:
        Exit code
    """
    try:
        acc_detector = AccelerationDetector()
        backends = acc_detector.detect_all()
        
        selector = BackendSelector()
        available_backends = selector.get_available_backends()
        
        if args.format == OutputFormat.JSON.value:
            # JSON output
            output = {
                "acceleration": {
                    backend.value: available
                    for backend, available in backends.items()
                },
                "backends": [b.value for b in available_backends]
            }
            print(json.dumps(output, indent=2))
        else:
            # Text output
            print("=== Available Backends ===\n")
            
            print("Acceleration:")
            for backend, available in backends.items():
                status = "✓" if available else "✗"
                print(f"  {status} {backend.value}")
            print()
            
            print("Inference Backends:")
            for backend in available_backends:
                print(f"  ✓ {backend.value}")
        
        return ExitCode.SUCCESS.value
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return ExitCode.ERROR.value


def cmd_test_backend(args) -> int:
    """
    Test backend selection for a model.
    
    Args:
        args: Parsed command arguments
        
    Returns:
        Exit code
    """
    try:
        model_type_str = args.model_type
        model_size = args.size
        
        # Parse model type
        try:
            model_type = ModelType(model_type_str)
        except ValueError:
            print(f"Error: Invalid model type '{model_type_str}'", file=sys.stderr)
            print(f"Valid types: {', '.join(t.value for t in ModelType)}", file=sys.stderr)
            return ExitCode.INVALID_ARGS.value
        
        selector = BackendSelector()
        result = selector.select_backend(
            model_type=model_type,
            model_size_gb=model_size
        )
        
        if args.format == OutputFormat.JSON.value:
            # JSON output
            output = {
                "backend": result.backend.value,
                "acceleration": result.acceleration.value,
                "gpu_index": result.gpu_index,
                "ngl": result.ngl,
                "context_size": result.context_size,
                "confidence": result.confidence,
                "reason": result.reason,
            }
            print(json.dumps(output, indent=2))
        else:
            # Text output
            print("=== Backend Selection Result ===\n")
            print(f"Model Type: {model_type.value}")
            if model_size:
                print(f"Model Size: {model_size:.1f} GB")
            print()
            print(f"Selected Backend: {result.backend.value}")
            print(f"Acceleration: {result.acceleration.value}")
            print(f"GPU Index: {result.gpu_index}")
            print(f"GPU Layers (ngl): {result.ngl}")
            print(f"Context Size: {result.context_size}")
            print(f"Confidence: {result.confidence:.0%}")
            print(f"\nReason: {result.reason}")
        
        return ExitCode.SUCCESS.value
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return ExitCode.ERROR.value


def cmd_ports(args) -> int:
    """
    Manage server ports.
    
    Args:
        args: Parsed command arguments
        
    Returns:
        Exit code
    """
    try:
        port_mgr = get_port_manager()
        
        if args.action == "list":
            allocations = port_mgr.get_all_allocations()
            
            if args.format == OutputFormat.JSON.value:
                # JSON output
                output = {
                    "allocations": [
                        {
                            "port": port,
                            "server_type": alloc.server_type.value,
                            "in_use": alloc.in_use,
                        }
                        for port, alloc in allocations.items()
                    ]
                }
                print(json.dumps(output, indent=2))
            else:
                # Text output
                if not allocations:
                    print("No ports allocated")
                else:
                    print("=== Port Allocations ===\n")
                    for port, alloc in allocations.items():
                        status = "in use" if alloc.in_use else "released"
                        print(f"Port {port}: {alloc.server_type.value} ({status})")
        
        elif args.action == "cleanup":
            cleaned = port_mgr.cleanup_dead_allocations()
            print(f"Cleaned up {cleaned} dead allocation(s)")
        
        return ExitCode.SUCCESS.value
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return ExitCode.ERROR.value


def main() -> int:
    """
    Main CLI entry point.
    
    Returns:
        Exit code
    """
    parser = argparse.ArgumentParser(
        description="TabAgent Native Host CLI",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s info                    # Show system information
  %(prog)s backends                # List available backends
  %(prog)s test bitnet_1.58 --size 3.5  # Test backend selection
  %(prog)s ports list              # List allocated ports
        """
    )
    
    # Global options
    parser.add_argument(
        '-v', '--verbose',
        action='count',
        default=0,
        help='Increase verbosity (can be repeated: -v, -vv, -vvv)'
    )
    parser.add_argument(
        '--format',
        choices=[f.value for f in OutputFormat],
        default=OutputFormat.TEXT.value,
        help='Output format'
    )
    
    # Subcommands
    subparsers = parser.add_subparsers(dest='command', help='Command to run')
    
    # Info command
    subparsers.add_parser(
        'info',
        help='Show system information'
    )
    
    # Backends command
    subparsers.add_parser(
        'backends',
        help='List available backends'
    )
    
    # Test backend command
    test_parser = subparsers.add_parser(
        'test',
        help='Test backend selection'
    )
    test_parser.add_argument(
        'model_type',
        help='Model type (bitnet_1.58, gguf_regular, etc.)'
    )
    test_parser.add_argument(
        '--size',
        type=float,
        help='Model size in GB'
    )
    
    # Ports command
    ports_parser = subparsers.add_parser(
        'ports',
        help='Manage server ports'
    )
    ports_parser.add_argument(
        'action',
        choices=['list', 'cleanup'],
        help='Port management action'
    )
    
    # Parse arguments
    args = parser.parse_args()
    
    # Setup logging
    setup_logging(args.verbose)
    
    # Route to command handler
    if not args.command:
        parser.print_help()
        return ExitCode.SUCCESS.value
    
    if args.command == 'info':
        return cmd_info(args)
    elif args.command == 'backends':
        return cmd_backends(args)
    elif args.command == 'test':
        return cmd_test_backend(args)
    elif args.command == 'ports':
        return cmd_ports(args)
    else:
        print(f"Error: Unknown command '{args.command}'", file=sys.stderr)
        return ExitCode.INVALID_ARGS.value


if __name__ == '__main__':
    sys.exit(main())

