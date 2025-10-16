# TabAgent Server API Reference

**Complete API documentation for all modules**

---

## Hardware Module

### `create_hardware_detector() -> HardwareDetector`

Factory function to create OS-specific hardware detector.

**Returns:** Platform-appropriate HardwareDetector instance

**Raises:** `NotImplementedError` if OS not supported

**Example:**
```python
from hardware import create_hardware_detector

detector = create_hardware_detector()
hw_info = detector.get_hardware_info()
print(f"GPUs: {len(hw_info.nvidia_gpus)}")
```

---

### `HardwareDetector`

Abstract base class for hardware detection.

#### Methods

**`get_hardware_info() -> HardwareInfo`**

Get complete hardware information.

**Returns:** HardwareInfo with all detected devices

**`get_cpu_info() -> CPUInfo`**

Get CPU information.

**`get_nvidia_gpus() -> List[GPUInfo]`**

Get NVIDIA GPU list with VRAM.

**`get_amd_gpus() -> List[GPUInfo]`**

Get AMD GPU list.

**`get_capabilities() -> HardwareCapabilities`**

Get acceleration capabilities (CUDA, Vulkan, etc).

---

### `BackendSelector`

Intelligent backend selection with VRAM awareness.

#### Constructor

```python
BackendSelector()
```

Automatically detects hardware on initialization.

#### Methods

**`select_backend(model_type, model_size_gb=None, model_layers=None, user_preference=None) -> BackendSelectionResult`**

Select optimal backend for a model.

**Parameters:**
- `model_type` (ModelType): Type of model
- `model_size_gb` (float, optional): Model size in GB
- `model_layers` (int, optional): Number of layers
- `user_preference` (BackendType, optional): User's preferred backend

**Returns:** BackendSelectionResult with configuration

**Example:**
```python
from hardware import BackendSelector
from core import ModelType

selector = BackendSelector()
result = selector.select_backend(
    model_type=ModelType.BITNET_158,
    model_size_gb=3.5
)

print(f"Backend: {result.backend}")
print(f"GPU layers: {result.ngl}")
print(f"Reason: {result.reason}")
```

---

### `GPULayerCalculator`

Static utility for calculating GPU layer offloading.

#### Methods

**`calculate_optimal_ngl(model_size_gb, vram_mb, context_size=4096, model_layers=None) -> int`**

Calculate optimal ngl value.

**Parameters:**
- `model_size_gb` (float): Model size in GB
- `vram_mb` (int): Available VRAM in MB
- `context_size` (int): Context size in tokens
- `model_layers` (int, optional): Total model layers

**Returns:** Number of layers to offload (0 if insufficient VRAM)

**Example:**
```python
from hardware import GPULayerCalculator

ngl = GPULayerCalculator.calculate_optimal_ngl(
    model_size_gb=7.0,
    vram_mb=8192,
    context_size=4096
)
print(f"Offload {ngl} layers")
```

---

## Server Management Module

### `get_port_manager() -> PortManager`

Get global PortManager singleton.

**Returns:** Global PortManager instance

**Example:**
```python
from server_mgmt import get_port_manager, ServerType

port_mgr = get_port_manager()
port = port_mgr.allocate_port(ServerType.BITNET_CPU)
print(f"Allocated port: {port}")
```

---

### `PortManager`

Manages port allocation for server processes.

#### Methods

**`allocate_port(server_type, preferred_port=None, force=False) -> int`**

Allocate a port for a server.

**Parameters:**
- `server_type` (ServerType): Server requesting port
- `preferred_port` (int, optional): Preferred port number
- `force` (bool): Skip in-use check

**Returns:** Allocated port number

**Raises:** `RuntimeError` if no ports available

**`release_port(port: int) -> bool`**

Release an allocated port.

**`get_all_allocations() -> Dict[int, PortAllocation]`**

Get all current allocations.

**`cleanup_dead_allocations() -> int`**

Clean up ports no longer in use.

**Returns:** Number of cleaned allocations

---

### `WrappedServer`

Subprocess wrapper with lifecycle management.

#### Constructor

```python
WrappedServer(config: ServerConfig)
```

**Parameters:**
- `config`: ServerConfig with executable, args, port, etc.

#### Methods

**`start() -> bool`**

Start the server process.

**Returns:** True if started successfully

**`stop(timeout=None) -> bool`**

Stop the server (graceful then force).

**`health_check() -> bool`**

Check if server is healthy.

**`wait_for_ready(timeout) -> bool`**

Wait for server to be ready.

**`is_running() -> bool`**

Check if process is alive.

#### Context Manager

```python
with WrappedServer(config) as server:
    server.wait_for_ready(30)
    # Use server
# Automatic cleanup
```

---

## Models Module

### `ModelLibrary`

Curated model library manager.

#### Constructor

```python
ModelLibrary(library_path=None)
```

Loads models from JSON file (default: `models/models_library.json`).

#### Methods

**`get_model(name: str) -> Optional[ModelInfo]`**

Get model by name.

**`list_models(model_type=None, use_case=None, recommended_only=False) -> List[ModelInfo]`**

List models with optional filtering.

**`search_models(query: str) -> List[ModelInfo]`**

Search by name or description.

**`get_recommended_models() -> List[ModelInfo]`**

Get recommended models.

**Example:**
```python
from models import ModelLibrary

library = ModelLibrary()

# Get recommended models
for model in library.get_recommended_models():
    print(f"{model.name} - {model.size_gb}GB")

# Search
results = library.search_models("code")
for model in results:
    print(f"{model.name}: {model.description}")
```

---

### `ModelManager`

Model management with download support.

#### Constructor

```python
ModelManager(library_path=None, cache_dir=None)
```

#### Methods

**`is_model_downloaded(model_name: str) -> bool`**

Check if model is downloaded.

**`download_model(model_name: str, variant=None) -> bool`**

Download model from HuggingFace.

**Requires:** `huggingface-hub` package

**`get_model_path(model_name: str) -> Optional[Path]`**

Get local path to downloaded model.

**`get_model_status(model_name: str) -> ModelStatus`**

Get download status.

**Example:**
```python
from models import ModelManager

mgr = ModelManager()

# Check status
if not mgr.is_model_downloaded("Phi-4"):
    print("Downloading...")
    mgr.download_model("Phi-4")

# Get path
path = mgr.get_model_path("Phi-4")
print(f"Model at: {path}")
```

---

## Core Types

### Enums

**`ModelType`**
- `BITNET_158` - BitNet 1.58-bit models
- `GGUF_REGULAR` - Standard GGUF models
- `SAFETENSORS` - Safetensors format
- `PYTORCH` - PyTorch .pt files

**`BackendType`**
- `BITNET_CPU` - BitNet CPU backend
- `BITNET_GPU` - BitNet GPU backend  
- `LMSTUDIO` - LM Studio backend

**`AccelerationBackend`**
- `CPU`, `CUDA`, `VULKAN`, `ROCM`, `METAL`, `DIRECTML`, `NPU`

**`ServerType`**
- `BITNET_CPU`, `BITNET_GPU`, `LMSTUDIO`, `WEBAPP`, `DEBUG`

**`ServerState`**
- `STOPPED`, `STARTING`, `RUNNING`, `STOPPING`, `ERROR`

---

### Pydantic Models

**`HardwareInfo`**
```python
class HardwareInfo(BaseModel):
    cpu: CPUInfo
    nvidia_gpus: List[GPUInfo]
    amd_gpus: List[GPUInfo]
    intel_gpus: List[GPUInfo]
    npu: Optional[NPUInfo]
    capabilities: HardwareCapabilities
    os_version: str
```

**`BackendSelectionResult`**
```python
@dataclass
class BackendSelectionResult:
    backend: BackendType
    acceleration: AccelerationBackend
    gpu_index: int
    ngl: int
    context_size: int
    confidence: float
    reason: str
```

**`ServerConfig`**
```python
@dataclass
class ServerConfig:
    executable: str
    args: List[str]
    port: int
    health_check_url: Optional[str]
    health_check_method: HealthCheckMethod
    startup_timeout: int
    health_check_interval: float
    graceful_shutdown_timeout: int
```

---

## CLI

### Commands

**`python cli.py info`**

Show system information (CPU, GPUs, capabilities).

**Options:**
- `--format json|text` - Output format
- `-v`, `-vv`, `-vvv` - Verbose levels

**`python cli.py backends`**

List available backends.

**`python cli.py test <model_type>`**

Test backend selection for a model.

**Options:**
- `--size <gb>` - Model size in GB

**`python cli.py ports list`**

List allocated ports.

**`python cli.py ports cleanup`**

Clean up dead port allocations.

### Examples

```bash
# System info
python cli.py info

# JSON output
python cli.py info --format json

# Test selection
python cli.py test bitnet_1.58 --size 3.5

# Verbose
python cli.py info -vv
```

---

## Error Handling

All public functions use proper exception handling:

```python
try:
    detector = create_hardware_detector()
    hw_info = detector.get_hardware_info()
except NotImplementedError:
    # OS not supported
    pass
except RuntimeError as e:
    # Hardware detection failed
    logger.error(f"Failed: {e}")
```

---

**Complete type safety. Production-ready APIs.** 🚀

