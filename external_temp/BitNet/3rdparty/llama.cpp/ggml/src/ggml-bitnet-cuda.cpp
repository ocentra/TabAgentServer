/**
 * BitNet CUDA Kernel Integration for llama.cpp
 * 
 * This file provides dynamic loading of the pre-built BitNet CUDA kernel
 * (libbitnet.dll on Windows, libbitnet.so on Linux) for W2A8 quantized inference.
 * 
 * The BitNet kernel is built separately using PyTorch's CUDA extension system
 * and provides optimized INT8xINT2 matrix multiplication for BitNet models.
 */

#include "ggml-bitnet-cuda.h"
#include "ggml-backend-impl.h"
#include "ggml.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Platform-specific dynamic library loading
#ifdef _WIN32
    #include <windows.h>
    #define LIBHANDLE HMODULE
    #define LOAD_LIBRARY(path) LoadLibraryA(path)
    #define GET_PROC_ADDRESS GetProcAddress
    #define CLOSE_LIBRARY FreeLibrary
    #define BITNET_LIB_NAME "libbitnet.dll"
#else
    #include <dlfcn.h>
    #define LIBHANDLE void*
    #define LOAD_LIBRARY(path) dlopen(path, RTLD_NOW | RTLD_LOCAL)
    #define GET_PROC_ADDRESS dlsym
    #define CLOSE_LIBRARY dlclose
    #define BITNET_LIB_NAME "libbitnet.so"
#endif

// CUDA types (to avoid requiring CUDA headers)
typedef void* cudaStream_t;
typedef unsigned short __nv_bfloat16;

// Function pointer type for BitNet CUDA kernel
typedef void (*bitlinear_int8xint2_func)(
    int8_t* input0,
    int8_t* input1,
    __nv_bfloat16* output0,
    __nv_bfloat16* s,
    __nv_bfloat16* ws,
    int M,
    int N,
    int K,
    cudaStream_t stream
);

// Global state
static LIBHANDLE g_bitnet_lib = NULL;
static bitlinear_int8xint2_func g_bitlinear_kernel = NULL;
static bool g_bitnet_initialized = false;
static bool g_bitnet_available = false;

//
// Initialization and cleanup
//

bool ggml_bitnet_cuda_init(void) {
    if (g_bitnet_initialized) {
        return g_bitnet_available;
    }

    g_bitnet_initialized = true;

    // Try to load the library
    // First, try from the same directory as the executable
    g_bitnet_lib = LOAD_LIBRARY(BITNET_LIB_NAME);

    if (!g_bitnet_lib) {
        // Try with explicit path (libs/windows/bitnet-kernel/ or libs/linux/bitnet-kernel/)
        #ifdef _WIN32
            g_bitnet_lib = LOAD_LIBRARY("libs/windows/bitnet-kernel/" BITNET_LIB_NAME);
        #else
            g_bitnet_lib = LOAD_LIBRARY("./libs/linux/bitnet-kernel/" BITNET_LIB_NAME);
        #endif
    }

    if (!g_bitnet_lib) {
        fprintf(stderr, "[BitNet] Warning: Could not load %s - BitNet GPU inference will not be available\n", BITNET_LIB_NAME);
        fprintf(stderr, "[BitNet] BitNet models will fall back to CPU inference\n");
        return false;
    }

    // Load the kernel function
    g_bitlinear_kernel = (bitlinear_int8xint2_func)GET_PROC_ADDRESS(g_bitnet_lib, "bitlinear_int8xint2");

    if (!g_bitlinear_kernel) {
        fprintf(stderr, "[BitNet] Error: Found %s but could not load 'bitlinear_int8xint2' function\n", BITNET_LIB_NAME);
        CLOSE_LIBRARY(g_bitnet_lib);
        g_bitnet_lib = NULL;
        return false;
    }

    g_bitnet_available = true;
    fprintf(stderr, "[BitNet] Successfully loaded BitNet CUDA kernel from %s\n", BITNET_LIB_NAME);
    return true;
}

void ggml_bitnet_cuda_free(void) {
    if (g_bitnet_lib) {
        CLOSE_LIBRARY(g_bitnet_lib);
        g_bitnet_lib = NULL;
        g_bitlinear_kernel = NULL;
    }
    g_bitnet_initialized = false;
    g_bitnet_available = false;
}

bool ggml_bitnet_cuda_is_available(void) {
    if (!g_bitnet_initialized) {
        ggml_bitnet_cuda_init();
    }
    return g_bitnet_available;
}

//
// Kernel invocation
//

bool ggml_bitnet_cuda_compute(
    int8_t* input,
    int8_t* weight,
    __nv_bfloat16* output,
    __nv_bfloat16* scale,
    __nv_bfloat16* weight_scale,
    int M,
    int N,
    int K,
    void* stream
) {
    if (!ggml_bitnet_cuda_is_available()) {
        return false;
    }

    // Call the kernel
    g_bitlinear_kernel(
        input,
        weight,
        output,
        scale,
        weight_scale,
        M,
        N,
        K,
        (cudaStream_t)stream
    );

    return true;
}

//
// Detection helpers
//

bool ggml_is_bitnet_model(const struct ggml_tensor* tensor) {
    // Check if this tensor uses BitNet quantization
    // BitNet models use special quantization types or metadata
    // This is a placeholder - actual detection should check model metadata
    
    if (!tensor) {
        return false;
    }

    // TODO: Check GGUF metadata for "bitnet" architecture or quantization type
    // For now, return false - detection will be added when integrating with llama.cpp
    return false;
}

bool ggml_should_use_bitnet_cuda(const struct ggml_tensor* tensor) {
    // Check if we should use BitNet CUDA kernel for this operation
    
    if (!ggml_bitnet_cuda_is_available()) {
        return false;
    }

    if (!ggml_is_bitnet_model(tensor)) {
        return false;
    }

    // Check if tensor is on CUDA device
    // TODO: Add device check when integrating with ggml-cuda backend
    
    return true;
}

