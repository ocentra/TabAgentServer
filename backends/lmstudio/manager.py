"""
LM Studio backend manager for inference.
Manages LM Studio server lifecycle and proxies requests to LM Studio API.
"""

import subprocess
import logging
import time
import requests
import platform
from pathlib import Path
from typing import Optional, Callable, Dict, Any, List

from core.message_types import (
    BackendType,
    ChatMessage,
    InferenceSettings,
    LoadingStatus
)


logger = logging.getLogger(__name__)


# Type aliases
ProgressCallback = Callable[[LoadingStatus, int, str], None]
StreamCallback = Callable[[str, Optional[str], int], None]


class LMStudioManager:
    """
    Manages LM Studio server lifecycle and API communication.
    Extension uses SDK directly, but native app ensures server is running.
    """
    
    def __init__(self):
        """Initialize LM Studio manager"""
        self._lms_path: Optional[Path] = None
        self._lmstudio_dir: Optional[Path] = None
        self._api_port: int = 1234
        self._api_host: str = "127.0.0.1"
        self._server_running: bool = False
        self._current_model: Optional[str] = None
        
        # Detect LM Studio installation
        self._detect_lmstudio()
        
        logger.info("LMStudioManager initialized")
    
    @property
    def is_installed(self) -> bool:
        """Check if LM Studio is installed"""
        return self._lms_path is not None and self._lms_path.exists()
    
    @property
    def is_bootstrapped(self) -> bool:
        """Check if LMS CLI is bootstrapped"""
        return self._lms_path is not None and self._lmstudio_dir is not None
    
    @property
    def is_server_running(self) -> bool:
        """Check if LM Studio API server is running"""
        return self._check_api_status()
    
    @property
    def current_model(self) -> Optional[str]:
        """Get currently loaded model"""
        return self._current_model
    
    def get_status(self) -> Dict[str, Any]:
        """
        Get comprehensive LM Studio status
        
        Returns:
            Dictionary with installation and runtime status
        """
        return {
            "installed": self.is_installed,
            "bootstrapped": self.is_bootstrapped,
            "server_running": self.is_server_running,
            "api_endpoint": f"http://{self._api_host}:{self._api_port}",
            "lms_path": str(self._lms_path) if self._lms_path else None,
            "current_model": self._current_model
        }
    
    def ensure_server_running(self) -> bool:
        """
        Ensure LM Studio server is running, start if needed
        
        Returns:
            True if server is running after this call, False otherwise
            
        Raises:
            RuntimeError: If LM Studio not installed or not bootstrapped
        """
        if not self.is_installed:
            raise RuntimeError("LM Studio is not installed")
        
        if not self.is_bootstrapped:
            raise RuntimeError("LM Studio CLI not bootstrapped. Run: lms bootstrap")
        
        # Check if already running
        if self._check_api_status():
            logger.info("LM Studio server already running")
            self._server_running = True
            return True
        
        # Start server
        logger.info("Starting LM Studio server")
        return self._start_server()
    
    def stop_server(self) -> bool:
        """
        Stop LM Studio server
        
        Returns:
            True if stopped successfully, False otherwise
        """
        if not self.is_bootstrapped:
            logger.warning("Cannot stop server - LMS CLI not bootstrapped")
            return False
        
        try:
            logger.info("Stopping LM Studio server")
            
            # Run: lms server stop
            result = subprocess.run(
                [str(self._lms_path), "server", "stop"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info("LM Studio server stopped successfully")
                self._server_running = False
                return True
            else:
                logger.warning(f"Failed to stop server: {result.stderr}")
                return False
                
        except Exception as e:
            logger.error(f"Error stopping LM Studio server: {e}")
            return False
    
    def proxy_chat_completion(
        self,
        messages: List[ChatMessage],
        settings: Optional[InferenceSettings] = None,
        stream_callback: Optional[StreamCallback] = None
    ) -> str:
        """
        Proxy chat completion request to LM Studio API
        
        Args:
            messages: Chat messages
            settings: Optional inference settings
            stream_callback: Optional streaming callback
            
        Returns:
            Generated text
            
        Raises:
            RuntimeError: If server not running or request fails
        """
        # Ensure server is running
        if not self.ensure_server_running():
            raise RuntimeError("LM Studio server is not running")
        
        # Build request payload (OpenAI-compatible)
        payload = {
            "messages": [
                {"role": msg.role.value, "content": msg.content}
                for msg in messages
            ],
            "temperature": settings.temperature if settings else 0.7,
            "max_tokens": settings.max_new_tokens if settings else 512,
            "top_p": settings.top_p if settings else 0.9,
            "stream": stream_callback is not None
        }
        
        # Call LM Studio API
        url = f"http://{self._api_host}:{self._api_port}/v1/chat/completions"
        
        try:
            if stream_callback:
                return self._stream_completion(url, payload, stream_callback)
            else:
                response = requests.post(url, json=payload, timeout=120)
                response.raise_for_status()
                
                result = response.json()
                return result["choices"][0]["message"]["content"]
                
        except requests.RequestException as e:
            logger.error(f"LM Studio API request failed: {e}")
            raise RuntimeError(f"Failed to get completion from LM Studio: {e}")
    
    def _detect_lmstudio(self) -> None:
        """Detect LM Studio installation"""
        # LM Studio directory
        lmstudio_dir = Path.home() / ".lmstudio"
        
        if platform.system() == "Windows":
            lms_path = lmstudio_dir / "bin" / "lms.exe"
        else:
            lms_path = lmstudio_dir / "bin" / "lms"
        
        if lms_path.exists():
            self._lms_path = lms_path
            self._lmstudio_dir = lmstudio_dir
            logger.info(f"LM Studio detected at: {lmstudio_dir}")
        else:
            logger.warning("LM Studio not found. Expected path: {lms_path}")
            self._lms_path = None
            self._lmstudio_dir = None
    
    def _check_api_status(self) -> bool:
        """
        Check if LM Studio API is responding
        
        Returns:
            True if API is reachable, False otherwise
        """
        try:
            url = f"http://{self._api_host}:{self._api_port}/v1/models"
            response = requests.get(url, timeout=2)
            return response.status_code == 200
        except requests.RequestException:
            return False
    
    def _start_server(self) -> bool:
        """
        Start LM Studio server using lms CLI
        
        Returns:
            True if started successfully, False otherwise
        """
        try:
            # Run: lms server start
            logger.info(f"Executing: {self._lms_path} server start")
            
            # Start in background (don't wait)
            subprocess.Popen(
                [str(self._lms_path), "server", "start"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL
            )
            
            # Wait for server to be ready (max 10 seconds)
            for i in range(20):  # 20 * 0.5s = 10s
                time.sleep(0.5)
                if self._check_api_status():
                    logger.info("LM Studio server started successfully")
                    self._server_running = True
                    return True
            
            logger.warning("LM Studio server did not become ready in time")
            return False
            
        except Exception as e:
            logger.error(f"Failed to start LM Studio server: {e}")
            return False
    
    def _stream_completion(
        self,
        url: str,
        payload: Dict[str, Any],
        stream_callback: StreamCallback
    ) -> str:
        """
        Handle streaming chat completion
        
        Args:
            url: API endpoint
            payload: Request payload
            stream_callback: Callback for tokens
            
        Returns:
            Complete generated text
        """
        full_text = ""
        token_count = 0
        
        try:
            response = requests.post(url, json=payload, stream=True, timeout=120)
            response.raise_for_status()
            
            for line in response.iter_lines():
                if line:
                    # Parse SSE format
                    if line.startswith(b"data: "):
                        import json
                        data_str = line[6:].decode('utf-8')
                        
                        if data_str.strip() == "[DONE]":
                            break
                        
                        try:
                            data = json.loads(data_str)
                            delta = data["choices"][0]["delta"]
                            token = delta.get("content", "")
                            
                            if token:
                                full_text += token
                                token_count += 1
                                stream_callback(token, None, token_count)
                                
                        except json.JSONDecodeError:
                            logger.debug(f"Skipping non-JSON line: {data_str}")
            
            return full_text
            
        except requests.RequestException as e:
            logger.error(f"Streaming completion failed: {e}")
            raise RuntimeError(f"Streaming failed: {e}")

