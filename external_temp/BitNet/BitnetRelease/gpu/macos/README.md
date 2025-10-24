# 🍎 macOS GPU Support (Metal Only)

## What Works on macOS

macOS supports **Metal GPU acceleration** for standard llama.cpp inference.

### ✅ Supported: Metal GPU
- **Technology**: Apple Metal (built into macOS)
- **Hardware**: ALL Macs
  - M1/M2/M3/M4 (Apple Silicon) - **Best performance!**
  - Intel Macs with Iris/AMD GPUs - **Good performance**
- **Location**: `standard-metal/`
- **Use Case**: Fast inference for standard GGUF models

### ❌ NOT Supported on macOS
- **CUDA**: Apple doesn't support NVIDIA GPUs
- **Vulkan**: Apple uses Metal instead
- **BitNet GPU**: Requires CUDA (Windows/Linux only)
- **OpenCL**: Deprecated by Apple

---

## Quick Start

### Standard GGUF Model with Metal GPU:

```bash
cd standard-metal/

# Full GPU offload (fastest)
./llama-server -m your-model.gguf -ngl 99

# Partial GPU offload (balance CPU/GPU)
./llama-server -m your-model.gguf -ngl 35

# CPU only (no GPU)
./llama-server -m your-model.gguf -ngl 0
```

### Performance Tips:

The `-ngl` flag controls how many layers to offload to GPU:
- **M1/M2/M3 Macs**: Use `-ngl 99` for full GPU (very fast!)
- **Intel Macs**: Use `-ngl 20-35` for balanced performance
- **Low VRAM**: Reduce `-ngl` value if you get memory errors

---

## Metal vs CUDA Performance

| Feature | CUDA (Win/Linux) | Metal (macOS) |
|---------|------------------|---------------|
| Speed | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Unified Memory | ❌ | ✅ (M1/M2/M3) |
| Power Efficiency | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| BitNet Support | ✅ | ❌ |

**M1/M2/M3 Macs have unified memory** - GPU can access full system RAM!

---

## For BitNet Inference on macOS

BitNet GPU kernels require CUDA, which is **NOT available on macOS**.

Instead, use **BitNet CPU** with ARM TL1 optimizations:

```bash
cd ../../cpu/macos/bitnet-arm/
./llama-server -m bitnet-model.gguf
```

BitNet ARM TL1 kernels provide **2-6x speedup** over standard GGUF on Apple Silicon!

---

## Build It Yourself

To rebuild Metal GPU binaries:

```bash
# From BitNet root directory
./build-all-macos.sh --variants metal
```

Or build everything:

```bash
./build-all-macos.sh
```

---

Build Date: $(date)

