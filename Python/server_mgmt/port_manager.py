"""
Smart port allocation and management for TabAgent server processes.

Provides conflict-free port allocation for multiple concurrent servers:
- BitNet CPU server
- BitNet GPU server  
- LM Studio server
- Future servers

Handles port availability checking, allocation tracking, and cleanup.
"""

import socket
import logging
from typing import Dict, Optional, Set
from enum import Enum
from dataclasses import dataclass, field


logger = logging.getLogger(__name__)


class ServerType(str, Enum):
    """Server types that need port allocation"""
    BITNET_CPU = "bitnet_cpu"
    BITNET_GPU = "bitnet_gpu"
    LMSTUDIO = "lmstudio"
    WEBAPP = "webapp"
    DEBUG = "debug"


class PortRange(int, Enum):
    """Default port range for allocation"""
    MIN = 8000
    MAX = 9000


class ReservedPort(int, Enum):
    """Reserved ports that should not be allocated"""
    LMSTUDIO_DEFAULT = 1234
    HTTP_ALT = 8080
    HTTP_PROXY = 8888


class DefaultPort(int, Enum):
    """Default preferred ports for each server type"""
    BITNET_CPU = 8081
    BITNET_GPU = 8082
    LMSTUDIO = 1234
    WEBAPP = 8000
    DEBUG = 8090


@dataclass
class PortAllocation:
    """
    Information about an allocated port.
    
    Attributes:
        port: Allocated port number
        server_type: Type of server using this port
        in_use: Whether port is currently active
        pid: Process ID using the port (if known)
    """
    port: int
    server_type: ServerType
    in_use: bool = True
    pid: Optional[int] = None


class PortManager:
    """
    Manages port allocation for TabAgent server processes.
    
    Features:
    - Conflict detection
    - Preferred port allocation
    - Multi-server support
    - Automatic cleanup
    - Thread-safe operations
    """
    
    def __init__(
        self,
        port_range_min: int = PortRange.MIN.value,
        port_range_max: int = PortRange.MAX.value
    ):
        """
        Initialize port manager.
        
        Args:
            port_range_min: Minimum port in allocation range
            port_range_max: Maximum port in allocation range
        """
        self.port_range_min = port_range_min
        self.port_range_max = port_range_max
        
        # Track allocated ports
        self._allocations: Dict[int, PortAllocation] = {}
        
        # Track reserved ports that should never be allocated
        self._reserved_ports: Set[int] = {
            port.value for port in ReservedPort
        }
        
        logger.info(
            f"PortManager initialized (range: {port_range_min}-{port_range_max}, "
            f"reserved: {len(self._reserved_ports)} ports)"
        )
    
    def allocate_port(
        self,
        server_type: ServerType,
        preferred_port: Optional[int] = None,
        force: bool = False
    ) -> int:
        """
        Allocate a port for a server.
        
        Args:
            server_type: Type of server requesting port
            preferred_port: Preferred port number (will try this first)
            force: If True, allocate even if port appears in use
            
        Returns:
            Allocated port number
            
        Raises:
            RuntimeError: If no ports available
        """
        # Check if server already has an allocation
        existing = self._find_allocation_by_server(server_type)
        if existing:
            logger.info(
                f"Server {server_type.value} already has port {existing.port}"
            )
            return existing.port
        
        # Try preferred port first
        if preferred_port is None:
            preferred_port = self._get_default_port(server_type)
        
        if self._is_port_available(preferred_port, force):
            return self._allocate_port_internal(
                port=preferred_port,
                server_type=server_type
            )
        
        # Preferred port not available, find next available
        logger.info(
            f"Preferred port {preferred_port} not available for "
            f"{server_type.value}, searching..."
        )
        
        available_port = self._find_available_port(force)
        if available_port is None:
            raise RuntimeError(
                f"No available ports in range "
                f"{self.port_range_min}-{self.port_range_max}"
            )
        
        return self._allocate_port_internal(
            port=available_port,
            server_type=server_type
        )
    
    def release_port(self, port: int) -> bool:
        """
        Release an allocated port.
        
        Args:
            port: Port number to release
            
        Returns:
            True if port was released, False if not allocated
        """
        if port not in self._allocations:
            logger.warning(f"Attempted to release unallocated port {port}")
            return False
        
        allocation = self._allocations[port]
        logger.info(
            f"Releasing port {port} ({allocation.server_type.value})"
        )
        del self._allocations[port]
        return True
    
    def release_server_ports(self, server_type: ServerType) -> int:
        """
        Release all ports allocated to a server type.
        
        Args:
            server_type: Server type to release ports for
            
        Returns:
            Number of ports released
        """
        ports_to_release = [
            port for port, alloc in self._allocations.items()
            if alloc.server_type == server_type
        ]
        
        for port in ports_to_release:
            self.release_port(port)
        
        if ports_to_release:
            logger.info(
                f"Released {len(ports_to_release)} port(s) for "
                f"{server_type.value}"
            )
        
        return len(ports_to_release)
    
    def get_port_for_server(self, server_type: ServerType) -> Optional[int]:
        """
        Get allocated port for a server type.
        
        Args:
            server_type: Server type to query
            
        Returns:
            Port number if allocated, None otherwise
        """
        allocation = self._find_allocation_by_server(server_type)
        return allocation.port if allocation else None
    
    def is_port_allocated(self, port: int) -> bool:
        """
        Check if a port is allocated by this manager.
        
        Args:
            port: Port number to check
            
        Returns:
            True if port is allocated
        """
        return port in self._allocations
    
    def get_all_allocations(self) -> Dict[int, PortAllocation]:
        """
        Get all current port allocations.
        
        Returns:
            Dictionary of port -> PortAllocation
        """
        return self._allocations.copy()
    
    def cleanup_dead_allocations(self) -> int:
        """
        Clean up allocations for ports that are no longer in use.
        
        Returns:
            Number of allocations cleaned up
        """
        dead_ports = []
        
        for port, allocation in self._allocations.items():
            if not self._is_port_in_use(port):
                logger.info(
                    f"Port {port} ({allocation.server_type.value}) "
                    f"is no longer in use"
                )
                dead_ports.append(port)
        
        for port in dead_ports:
            del self._allocations[port]
        
        if dead_ports:
            logger.info(f"Cleaned up {len(dead_ports)} dead port allocation(s)")
        
        return len(dead_ports)
    
    def _allocate_port_internal(
        self,
        port: int,
        server_type: ServerType
    ) -> int:
        """
        Internal method to allocate a port.
        
        Args:
            port: Port number to allocate
            server_type: Server type allocating the port
            
        Returns:
            Allocated port number
        """
        allocation = PortAllocation(
            port=port,
            server_type=server_type,
            in_use=True
        )
        
        self._allocations[port] = allocation
        logger.info(f"Allocated port {port} to {server_type.value}")
        return port
    
    def _find_allocation_by_server(
        self,
        server_type: ServerType
    ) -> Optional[PortAllocation]:
        """
        Find allocation for a server type.
        
        Args:
            server_type: Server type to search for
            
        Returns:
            PortAllocation if found, None otherwise
        """
        for allocation in self._allocations.values():
            if allocation.server_type == server_type:
                return allocation
        return None
    
    def _get_default_port(self, server_type: ServerType) -> int:
        """
        Get default preferred port for server type.
        
        Args:
            server_type: Server type
            
        Returns:
            Default port number
        """
        port_map = {
            ServerType.BITNET_CPU: DefaultPort.BITNET_CPU.value,
            ServerType.BITNET_GPU: DefaultPort.BITNET_GPU.value,
            ServerType.LMSTUDIO: DefaultPort.LMSTUDIO.value,
            ServerType.WEBAPP: DefaultPort.WEBAPP.value,
            ServerType.DEBUG: DefaultPort.DEBUG.value,
        }
        
        return port_map.get(server_type, PortRange.MIN.value)
    
    def _is_port_available(self, port: int, force: bool = False) -> bool:
        """
        Check if a port is available for allocation.
        
        Args:
            port: Port number to check
            force: If True, skip in-use check
            
        Returns:
            True if port is available
        """
        # Check if port is in valid range
        if not (self.port_range_min <= port <= self.port_range_max):
            # Allow reserved ports outside range (like LM Studio 1234)
            if port not in [p.value for p in DefaultPort]:
                logger.debug(f"Port {port} outside valid range")
                return False
        
        # Check if port is reserved
        if port in self._reserved_ports:
            # LM Studio port is reserved but can be allocated if specifically requested
            if port == ReservedPort.LMSTUDIO_DEFAULT.value:
                pass  # Allow LM Studio default port
            else:
                logger.debug(f"Port {port} is reserved")
                return False
        
        # Check if already allocated
        if port in self._allocations:
            logger.debug(f"Port {port} already allocated")
            return False
        
        # Check if port is actually in use (unless forced)
        if not force and self._is_port_in_use(port):
            logger.debug(f"Port {port} is in use by external process")
            return False
        
        return True
    
    def _find_available_port(self, force: bool = False) -> Optional[int]:
        """
        Find next available port in range.
        
        Args:
            force: If True, skip in-use check
            
        Returns:
            Available port number or None
        """
        for port in range(self.port_range_min, self.port_range_max + 1):
            if self._is_port_available(port, force):
                return port
        return None
    
    @staticmethod
    def _is_port_in_use(port: int) -> bool:
        """
        Check if a port is currently in use.
        
        Args:
            port: Port number to check
            
        Returns:
            True if port is in use
        """
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
                sock.settimeout(0.1)
                result = sock.connect_ex(('127.0.0.1', port))
                # Port is in use if connection succeeds (result == 0)
                return result == 0
        except Exception as e:
            logger.debug(f"Error checking port {port}: {e}")
            # If we can't check, assume it's available
            return False


# Singleton instance for global port management
_global_port_manager: Optional[PortManager] = None


def get_port_manager() -> PortManager:
    """
    Get global PortManager instance.
    
    Returns:
        Global PortManager singleton
    """
    global _global_port_manager
    if _global_port_manager is None:
        _global_port_manager = PortManager()
    return _global_port_manager


def reset_port_manager() -> None:
    """
    Reset global PortManager instance.
    
    Useful for testing or cleanup.
    """
    global _global_port_manager
    _global_port_manager = None

