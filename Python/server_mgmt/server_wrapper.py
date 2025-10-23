"""
Robust server process lifecycle management for TabAgent.

Provides a wrapper for managing subprocess-based servers with:
- Health checking
- Graceful shutdown
- Automatic cleanup
- Context manager support
- Process monitoring
"""

import subprocess
import time
import logging
import requests
import signal
from typing import List, Optional, Callable
from enum import Enum
from dataclasses import dataclass
from pathlib import Path


logger = logging.getLogger(__name__)


class ServerState(str, Enum):
    """Server process states"""
    STOPPED = "stopped"
    STARTING = "starting"
    RUNNING = "running"
    STOPPING = "stopping"
    ERROR = "error"


class HealthCheckMethod(str, Enum):
    """Health check methods"""
    HTTP_GET = "http_get"
    HTTP_POST = "http_post"
    TCP_CONNECT = "tcp_connect"
    PROCESS_ALIVE = "process_alive"


class ShutdownMethod(str, Enum):
    """Shutdown methods"""
    SIGTERM = "sigterm"
    SIGINT = "sigint"
    SIGKILL = "sigkill"


class TimeoutValue(int, Enum):
    """Timeout values in seconds"""
    STARTUP = 30
    HEALTH_CHECK = 2
    GRACEFUL_SHUTDOWN = 5
    FORCED_SHUTDOWN = 2


@dataclass
class ServerConfig:
    """
    Configuration for wrapped server.
    
    Attributes:
        executable: Path to server executable
        args: Command line arguments
        port: Server port
        health_check_url: URL for health checking (if HTTP)
        health_check_method: Method to use for health checking
        startup_timeout: Seconds to wait for startup
        health_check_interval: Seconds between health checks
        graceful_shutdown_timeout: Seconds to wait for graceful shutdown
    """
    executable: str
    args: List[str]
    port: int
    health_check_url: Optional[str] = None
    health_check_method: HealthCheckMethod = HealthCheckMethod.HTTP_GET
    startup_timeout: int = TimeoutValue.STARTUP.value
    health_check_interval: float = 1.0
    graceful_shutdown_timeout: int = TimeoutValue.GRACEFUL_SHUTDOWN.value


class WrappedServer:
    """
    Wrapper for server subprocess with lifecycle management.
    
    Provides:
    - Process spawning and monitoring
    - Health checking
    - Graceful shutdown with fallback to force kill
    - Context manager support
    - Automatic cleanup
    
    Example:
        config = ServerConfig(
            executable="llama-server",
            args=["--port", "8081"],
            port=8081,
            health_check_url="http://localhost:8081/health"
        )
        
        with WrappedServer(config) as server:
            # Server is running
            server.wait_for_ready()
            # Use server
        # Server automatically stopped
    """
    
    def __init__(self, config: ServerConfig):
        """
        Initialize wrapped server.
        
        Args:
            config: Server configuration
        """
        self.config = config
        self.process: Optional[subprocess.Popen] = None
        self.state: ServerState = ServerState.STOPPED
        self._last_health_check: Optional[float] = None
        
        logger.info(
            f"WrappedServer initialized for {config.executable} on port {config.port}"
        )
    
    def start(self) -> bool:
        """
        Start the server process.
        
        Returns:
            True if server started successfully
            
        Raises:
            RuntimeError: If server is already running
            FileNotFoundError: If executable not found
        """
        if self.state != ServerState.STOPPED:
            raise RuntimeError(
                f"Cannot start server in state {self.state.value}"
            )
        
        executable_path = Path(self.config.executable)
        if not executable_path.exists():
            raise FileNotFoundError(
                f"Server executable not found: {self.config.executable}"
            )
        
        self.state = ServerState.STARTING
        logger.info(f"Starting server: {self.config.executable} {' '.join(self.config.args)}")
        
        try:
            # Build full command
            command = [self.config.executable] + self.config.args
            
            # Start process
            self.process = subprocess.Popen(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                stdin=subprocess.DEVNULL,
                text=True,
                bufsize=1
            )
            
            logger.info(
                f"Server process started (PID: {self.process.pid})"
            )
            
            # Wait for server to be ready
            if self.wait_for_ready(self.config.startup_timeout):
                self.state = ServerState.RUNNING
                logger.info(f"Server is ready on port {self.config.port}")
                return True
            else:
                logger.error("Server failed to become ready within timeout")
                self.stop()
                self.state = ServerState.ERROR
                return False
                
        except Exception as e:
            logger.error(f"Failed to start server: {e}")
            self.state = ServerState.ERROR
            if self.process:
                self.process.kill()
                self.process = None
            return False
    
    def stop(self, timeout: Optional[int] = None) -> bool:
        """
        Stop the server process.
        
        Attempts graceful shutdown first, then force kills if needed.
        
        Args:
            timeout: Seconds to wait for graceful shutdown
            
        Returns:
            True if server stopped successfully
        """
        if self.state == ServerState.STOPPED:
            logger.info("Server already stopped")
            return True
        
        if self.process is None:
            self.state = ServerState.STOPPED
            return True
        
        if timeout is None:
            timeout = self.config.graceful_shutdown_timeout
        
        self.state = ServerState.STOPPING
        logger.info(f"Stopping server (PID: {self.process.pid})")
        
        # Try graceful shutdown (SIGTERM)
        try:
            self.process.terminate()
            logger.debug(f"Sent SIGTERM to process {self.process.pid}")
            
            # Wait for graceful shutdown
            try:
                self.process.wait(timeout=timeout)
                logger.info("Server stopped gracefully")
                self.state = ServerState.STOPPED
                self.process = None
                return True
            except subprocess.TimeoutExpired:
                logger.warning(
                    f"Server did not stop gracefully within {timeout}s, "
                    f"force killing"
                )
        except Exception as e:
            logger.warning(f"Error during graceful shutdown: {e}")
        
        # Force kill (SIGKILL)
        try:
            self.process.kill()
            logger.debug(f"Sent SIGKILL to process {self.process.pid}")
            
            # Wait for force kill
            self.process.wait(timeout=TimeoutValue.FORCED_SHUTDOWN.value)
            logger.info("Server force killed")
            self.state = ServerState.STOPPED
            self.process = None
            return True
            
        except Exception as e:
            logger.error(f"Failed to kill server process: {e}")
            self.state = ServerState.ERROR
            return False
    
    def restart(self) -> bool:
        """
        Restart the server.
        
        Returns:
            True if restart successful
        """
        logger.info("Restarting server")
        self.stop()
        time.sleep(1.0)  # Brief pause between stop and start
        return self.start()
    
    def is_running(self) -> bool:
        """
        Check if server process is running.
        
        Returns:
            True if process is alive
        """
        if self.process is None:
            return False
        
        # Check if process is still alive
        poll_result = self.process.poll()
        is_alive = poll_result is None
        
        if not is_alive and self.state == ServerState.RUNNING:
            logger.warning(
                f"Server process died unexpectedly (exit code: {poll_result})"
            )
            self.state = ServerState.ERROR
        
        return is_alive
    
    def health_check(self) -> bool:
        """
        Perform health check on server.
        
        Returns:
            True if server is healthy
        """
        # First check if process is alive
        if not self.is_running():
            return False
        
        method = self.config.health_check_method
        
        if method == HealthCheckMethod.PROCESS_ALIVE:
            # Just check if process is running
            return True
        
        elif method in [HealthCheckMethod.HTTP_GET, HealthCheckMethod.HTTP_POST]:
            # HTTP health check
            if not self.config.health_check_url:
                logger.warning("No health check URL configured")
                return self.is_running()
            
            try:
                if method == HealthCheckMethod.HTTP_GET:
                    response = requests.get(
                        self.config.health_check_url,
                        timeout=TimeoutValue.HEALTH_CHECK.value
                    )
                else:
                    response = requests.post(
                        self.config.health_check_url,
                        timeout=TimeoutValue.HEALTH_CHECK.value
                    )
                
                is_healthy = response.status_code == 200
                self._last_health_check = time.time()
                return is_healthy
                
            except requests.exceptions.RequestException as e:
                logger.debug(f"Health check failed: {e}")
                return False
        
        elif method == HealthCheckMethod.TCP_CONNECT:
            # TCP connection check
            import socket
            try:
                with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
                    sock.settimeout(TimeoutValue.HEALTH_CHECK.value)
                    result = sock.connect_ex(('127.0.0.1', self.config.port))
                    is_connected = result == 0
                    self._last_health_check = time.time()
                    return is_connected
            except Exception as e:
                logger.debug(f"TCP health check failed: {e}")
                return False
        
        # Unknown method
        logger.warning(f"Unknown health check method: {method.value}")
        return self.is_running()
    
    def wait_for_ready(self, timeout: int) -> bool:
        """
        Wait for server to become ready.
        
        Args:
            timeout: Seconds to wait
            
        Returns:
            True if server became ready within timeout
        """
        logger.info(f"Waiting for server to be ready (timeout: {timeout}s)")
        
        start_time = time.time()
        interval = self.config.health_check_interval
        
        while (time.time() - start_time) < timeout:
            if self.health_check():
                elapsed = time.time() - start_time
                logger.info(f"Server ready after {elapsed:.1f}s")
                return True
            
            # Check if process died
            if not self.is_running():
                logger.error("Server process died during startup")
                return False
            
            time.sleep(interval)
        
        logger.error(f"Server not ready after {timeout}s")
        return False
    
    def get_pid(self) -> Optional[int]:
        """
        Get server process ID.
        
        Returns:
            Process ID if running, None otherwise
        """
        return self.process.pid if self.process else None
    
    def get_state(self) -> ServerState:
        """
        Get current server state.
        
        Returns:
            Current ServerState
        """
        return self.state
    
    def __enter__(self):
        """Context manager entry"""
        self.start()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        self.stop()
        return False
    
    def __del__(self):
        """Destructor - ensure cleanup"""
        if self.process and self.is_running():
            logger.warning("WrappedServer destroyed while running, cleaning up")
            self.stop()

