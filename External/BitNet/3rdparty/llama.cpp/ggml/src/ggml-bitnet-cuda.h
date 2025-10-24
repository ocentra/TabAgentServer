/**
 * BitNet CUDA Kernel Integration Header
 * 
 * Provides interface for dynamically loading and using the pre-built
 * BitNet CUDA kernel (libbitnet.dll/libbitnet.so) within llama.cpp
 */

#pragma once

#include "ggml.h"
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Initialize BitNet CUDA kernel (loads libbitnet.dll/.so)
 * @return true if successful, false otherwise
 */
bool ggml_bitnet_cuda_init(void);

/**
 * Free BitNet CUDA resources and unload library
 */
void ggml_bitnet_cuda_free(void);

/**
 * Check if BitNet CUDA kernel is available
 * @return true if libbitnet.dll/.so was loaded successfully
 */
bool ggml_bitnet_cuda_is_available(void);

/**
 * Execute BitNet CUDA kernel (INT8xINT2 matrix multiplication)
 * @param input Input tensor (INT8)
 * @param weight Weight tensor (INT2, packed)
 * @param output Output tensor (bfloat16)
 * @param scale Activation scale (bfloat16)
 * @param weight_scale Weight scale (bfloat16)
 * @param M Batch size
 * @param N Output features
 * @param K Input features
 * @param stream CUDA stream (void* to avoid requiring cuda.h)
 * @return true if kernel was executed, false otherwise
 */
bool ggml_bitnet_cuda_compute(
    int8_t* input,
    int8_t* weight,
    unsigned short* output,  // __nv_bfloat16
    unsigned short* scale,
    unsigned short* weight_scale,
    int M,
    int N,
    int K,
    void* stream
);

/**
 * Check if a tensor belongs to a BitNet model
 * @param tensor Tensor to check
 * @return true if tensor is from a BitNet model
 */
bool ggml_is_bitnet_model(const struct ggml_tensor* tensor);

/**
 * Check if BitNet CUDA kernel should be used for this tensor
 * @param tensor Tensor to check
 * @return true if BitNet CUDA should be used
 */
bool ggml_should_use_bitnet_cuda(const struct ggml_tensor* tensor);

#ifdef __cplusplus
}
#endif

