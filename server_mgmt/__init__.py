"""
Server process management for TabAgent.

Provides port allocation and server lifecycle management
for multiple concurrent server processes.
"""

from .port_manager import (
    PortManager,
    ServerType,
    PortAllocation,
    get_port_manager,
    reset_port_manager,
)

from .server_wrapper import (
    WrappedServer,
    ServerConfig,
    ServerState,
    HealthCheckMethod,
    ShutdownMethod,
)

__all__ = [
    # Port management
    'PortManager',
    'ServerType',
    'PortAllocation',
    'get_port_manager',
    'reset_port_manager',
    
    # Server wrapper
    'WrappedServer',
    'ServerConfig',
    'ServerState',
    'HealthCheckMethod',
    'ShutdownMethod',
]

