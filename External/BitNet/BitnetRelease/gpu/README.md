# GPU Modules Directory

This directory contains Python modules and CUDA kernels for GPU inference.

## Structure

```
gpu/
├── windows/
│   ├── bitlinear_cuda.pyd       # CUDA kernel (compiled)
│   ├── model.py                 # Model implementation
│   ├── generate.py              # Generation logic (FastGen)
│   ├── tokenizer.py             # Tokenizer
│   ├── tokenizer.model          # Tokenizer data (2.1 MB)
│   ├── pack_weight.py           # Weight packing
│   ├── sample_utils.py          # Sampling utilities
│   ├── stats.py                 # Statistics
│   └── convert_*.py             # Conversion scripts
├── macos/
│   └── (Python files only - no CUDA support on macOS)
└── linux/
    ├── bitlinear_cuda.so        # CUDA kernel (compiled)
    └── (same Python files as Windows)
```

## What is GPU Inference?

GPU inference uses CUDA kernels directly for 2-6x faster inference than CPU:

**Requirements:**
- NVIDIA GPU (RTX, Tesla, etc.)
- CUDA 12.1+ (installed via NVIDIA drivers)
- PyTorch with CUDA support
- BitNet `.pt` checkpoint files (not .gguf)

**Advantages:**
- 100-150 tok/s on RTX 4090 (vs 15-25 tok/s CPU)
- Lower latency
- Direct Python API (no HTTP overhead)

## Build Instructions

### Windows (CUDA):

**Prerequisites:**
- CUDA Toolkit 12.1+
- Visual Studio 2022 with C++ tools
- Python 3.9-3.11

**Steps:**
```cmd
cd E:\Desktop\TabAgent\Server\BitNet\gpu\bitnet_kernels

REM Install PyTorch with CUDA
python -m pip install torch --index-url https://download.pytorch.org/whl/cu121

REM Build kernel
python setup.py build_ext --inplace

REM Copy to Release
mkdir ..\..\Release\gpu\windows
copy *.pyd ..\..\Release\gpu\windows\
cd ..
copy *.py ..\Release\gpu\windows\
copy tokenizer.model ..\Release\gpu\windows\
```

### Linux (CUDA):

**Prerequisites:**
- CUDA Toolkit 12.1+
- GCC/G++ compiler
- Python 3.9-3.11

**Steps:**
```bash
cd gpu/bitnet_kernels

# Install PyTorch with CUDA
python -m pip install torch --index-url https://download.pytorch.org/whl/cu121

# Build kernel
python setup.py build_ext --inplace

# Copy to Release
mkdir -p ../../Release/gpu/linux
cp *.so ../../Release/gpu/linux/
cd ..
cp *.py ../Release/gpu/linux/
cp tokenizer.model ../Release/gpu/linux/
```

### macOS (No CUDA - Placeholder):

```bash
cd gpu

# Copy Python files only (no kernel)
mkdir -p ../Release/gpu/macos
cp *.py ../Release/gpu/macos/
cp tokenizer.model ../Release/gpu/macos/

# Create README explaining limitation
cat > ../Release/gpu/macos/README.txt << 'EOF'
macOS GPU Support Limitation
============================

macOS does not support NVIDIA CUDA.
GPU acceleration is only available on:
- Windows with NVIDIA GPU
- Linux with NVIDIA GPU

On macOS, use CPU backend (ARM TL1 optimized).
EOF
```

## Usage

### From Python:

```python
import sys
sys.path.insert(0, 'Release/gpu/windows')  # or linux

from generate import FastGen, GenArgs

# Build model
gen = FastGen.build(
    ckpt_dir="path/to/checkpoints/",
    gen_args=GenArgs(),
    device="cuda:0"
)

# Generate
tokens = gen.tokenizer.encode("Hello, world!", bos=False, eos=False)
stats, out_tokens = gen.generate_all(
    prompts=[tokens],
    use_cuda_graphs=True,
    use_sampling=True
)

# Decode
text = gen.tokenizer.decode(out_tokens[0])
print(text)
```

### From TabAgent:

TabAgent's `BitNetManager` automatically uses GPU if:
1. CUDA is available (`torch.cuda.is_available()`)
2. User loads a `.pt` checkpoint directory
3. `prefer_gpu=True` (default)

```python
# TabAgent integration
from backends.bitnet import BitNetManager

manager = BitNetManager()
manager.load_model("checkpoints/", prefer_gpu=True)  # Uses GPU
result = manager.generate(messages)
```

## Model Format

**GPU requires .pt checkpoints:**
```
checkpoints/
├── model_state_fp16.pt    # FP16 weights for prefill
├── model_state_int2.pt    # INT2 weights for decode
└── tokenizer.model        # Tokenizer (optional)
```

**CPU uses .gguf files:**
```
model.gguf                 # Single file
```

## Testing

### Test Kernel Import:
```python
# Windows
import bitlinear_cuda
print("✅ Windows CUDA kernel loaded")

# Linux
import bitlinear_cuda
print("✅ Linux CUDA kernel loaded")
```

### Test GPU Generation:
```bash
cd Release/gpu/windows  # or linux
python ../../../gpu/test.py  # Run test script
```

## Integration with TabAgent

TabAgent copies these files to `Server/backends/bitnet/gpu/{platform}/`:

```
Server/backends/bitnet/gpu/
├── windows/
│   ├── bitlinear_cuda.pyd  ← From Release/gpu/windows/
│   └── *.py                ← From Release/gpu/windows/
├── macos/
│   └── *.py                ← From Release/gpu/macos/
└── linux/
    ├── bitlinear_cuda.so   ← From Release/gpu/linux/
    └── *.py                ← From Release/gpu/linux/
```

When loading a BitNet model on GPU:
1. Manager detects platform
2. Adds `backends/bitnet/gpu/{platform}/` to sys.path
3. Imports `generate.py` and `model.py`
4. Kernel (.pyd/.so) is loaded automatically via `ctypes`

## Performance

| Hardware | Speed | Memory | Notes |
|----------|-------|--------|-------|
| RTX 4090 | ~100-150 tok/s | 2-3 GB VRAM | Best performance |
| RTX 3090 | ~80-120 tok/s | 2-3 GB VRAM | Excellent |
| RTX 3080 | ~60-90 tok/s | 2-3 GB VRAM | Great |
| RTX 2080 Ti | ~40-60 tok/s | 2-3 GB VRAM | Good |

*Speed for 2B model; scales with model size*

## File Sizes

- **bitlinear_cuda.pyd/so:** ~5-15 MB
- **Python files:** ~100-200 KB total
- **tokenizer.model:** ~2.1 MB

## Troubleshooting

### CUDA Not Available
```python
import torch
print(torch.cuda.is_available())  # Should be True
```
**Solution:** Install CUDA-enabled PyTorch

### Kernel Build Fails
```
error: command 'nvcc' failed
```
**Solution:** Install CUDA Toolkit and add to PATH

### Import Error
```
ImportError: DLL load failed
```
**Solution (Windows):** Ensure CUDA DLLs are in PATH

### Out of Memory
```
CUDA out of memory
```
**Solution:** Use smaller model or reduce batch size

## Notes

- **macOS:** No CUDA support (use CPU backend)
- **CUDA 12.1+:** Required for compatibility
- **PyTorch:** Must be built with CUDA support
- **Checkpoints:** Use .pt format (not .gguf) for GPU

