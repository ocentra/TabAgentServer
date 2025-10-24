# 🔍 BitNet Linux GPU Build Verification Report

## Build Matrix

This directory contains **3 GPU variants**:

### Standard GPU
- **standard-cuda-vulkan/** - CUDA 12.x accelerated (NVIDIA GPUs)
- **standard-opencl/** - OpenCL universal GPU (AMD, Intel, NVIDIA)

### BitNet GPU
- **bitnet-python-cuda/** - BitNet Python CUDA kernels (custom 1.58-bit)

## Quick Start

### Standard GPU (CUDA):
```bash
cd standard-cuda-vulkan/
./llama-server -m model.gguf -ngl 35  # Offload 35 layers to GPU
```

### BitNet GPU (Python CUDA):
```bash
cd bitnet-python-cuda/
source ../../bitnet-gpu-env-linux/bin/activate
python generate.py --checkpoint <model-path>
```

## GPU Layer Offloading

The `-ngl` flag controls how many layers run on GPU:

```bash
# Full GPU (fastest)
./llama-server -ngl 99 -m model.gguf

# Partial GPU (balance CPU/GPU)
./llama-server -ngl 20 -m model.gguf

# CPU only
./llama-server -ngl 0 -m model.gguf
```

## Technical Details

### Standard CUDA+Vulkan:
- **CUDA:** 12.1+ required
- **Compute Capability:** 7.5, 8.0, 8.6, 8.9, 9.0
- **Multi-GPU:** Supported via `-sm` flag
- **Vulkan:** Requires Ubuntu 22.04+ for best support

### BitNet Python CUDA:
- **Python:** 3.9-3.11 (NOT 3.12+)
- **PyTorch:** 2.3.1+cu121
- **xformers:** 0.0.27
- **Custom Kernels:** `libbitnet.so` + `bitlinear_cuda.so`

## File Counts

- **bitnet-python-cuda/**: 15 files
- **standard-cuda-vulkan/**: 56 files
- **standard-opencl/**: 55 files

---
Build Date: Tue Oct 21 21:18:33 EDT 2025
