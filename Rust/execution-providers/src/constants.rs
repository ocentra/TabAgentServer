//! Configuration key constants for all execution providers
//!
//! This module contains all configuration keys as constants to avoid string literals
//! throughout the codebase, following the project's "no string literals" policy.

// ============================================================================
// Common Configuration Keys (used across multiple providers)
// ============================================================================

pub const DEVICE_ID: &str = "device_id";
pub const USE_ARENA: &str = "use_arena";
pub const ARENA_EXTEND_STRATEGY: &str = "arena_extend_strategy";
pub const GPU_MEM_LIMIT: &str = "gpu_mem_limit";

// ============================================================================
// CPU Execution Provider
// ============================================================================

pub const ENABLE_CPU_MEM_ARENA: &str = "enable_cpu_mem_arena";

// ============================================================================
// CUDA Execution Provider
// ============================================================================

pub const CUDNN_CONV_ALGO_SEARCH: &str = "cudnn_conv_algo_search";
pub const CUDNN_CONV_USE_MAX_WORKSPACE: &str = "cudnn_conv_use_max_workspace";
pub const CUDNN_CONV1D_PAD_TO_NC1D: &str = "cudnn_conv1d_pad_to_nc1d";
pub const ENABLE_CUDA_GRAPH: &str = "enable_cuda_graph";
pub const ENABLE_SKIP_LAYER_NORM_STRICT_MODE: &str = "enable_skip_layer_norm_strict_mode";
pub const USE_TF32: &str = "use_tf32";
pub const PREFER_NHWC: &str = "prefer_nhwc";
pub const USER_COMPUTE_STREAM: &str = "user_compute_stream";
pub const SDPA_KERNEL: &str = "sdpa_kernel";
pub const FUSE_CONV_BIAS: &str = "fuse_conv_bias";
pub const DO_COPY_IN_DEFAULT_STREAM: &str = "do_copy_in_default_stream";
pub const ENABLE_CUDA_GRAPH_CAPTURE: &str = "enable_cuda_graph_capture";
pub const CUDA_STREAM_PRIORITY: &str = "cuda_stream_priority";
pub const ENABLE_CUDA_GRAPH_CONDITIONAL: &str = "enable_cuda_graph_conditional";
pub const TUNABLE_OP_ENABLE: &str = "tunable_op_enable";
pub const TUNABLE_OP_TUNING_ENABLE: &str = "tunable_op_tuning_enable";
pub const TUNABLE_OP_MAX_TUNING_DURATION_MS: &str = "tunable_op_max_tuning_duration_ms";
pub const ENABLE_CUDA_MEM_POOL: &str = "enable_cuda_mem_pool";
pub const DEFAULT_MEMORY_ARENA_CFG: &str = "default_memory_arena_cfg";
pub const CUDNN_CONV_ALGORITHM: &str = "cudnn_conv_algorithm";
pub const USE_EP_LEVEL_UNIFIED_STREAM: &str = "use_ep_level_unified_stream";
pub const CUDNN_RNN_MODE: &str = "cudnn_rnn_mode";
pub const CUDNN_BN_SPATIAL_PERSISTENT: &str = "cudnn_bn_spatial_persistent";
pub const ENABLE_MEM_PATTERN: &str = "enable_mem_pattern";
pub const USE_CUDNN_FRONTEND: &str = "use_cudnn_frontend";
pub const FUSE_BN_RELU: &str = "fuse_bn_relu";
pub const FUSE_BN_ADD_RELU: &str = "fuse_bn_add_relu";

// ============================================================================
// TensorRT Execution Provider
// ============================================================================

pub const TRT_MAX_WORKSPACE_SIZE: &str = "trt_max_workspace_size";
pub const TRT_MIN_SUBGRAPH_SIZE: &str = "trt_min_subgraph_size";
pub const TRT_MAX_PARTITION_ITERATIONS: &str = "trt_max_partition_iterations";
pub const TRT_FP16_ENABLE: &str = "trt_fp16_enable";
pub const TRT_INT8_ENABLE: &str = "trt_int8_enable";
pub const TRT_INT8_CALIBRATION_TABLE_NAME: &str = "trt_int8_calibration_table_name";
pub const TRT_INT8_USE_NATIVE_CALIBRATION_TABLE: &str = "trt_int8_use_native_calibration_table";
pub const TRT_DLA_ENABLE: &str = "trt_dla_enable";
pub const TRT_DLA_CORE: &str = "trt_dla_core";
pub const TRT_ENGINE_CACHE_ENABLE: &str = "trt_engine_cache_enable";
pub const TRT_ENGINE_CACHE_PATH: &str = "trt_engine_cache_path";
pub const TRT_ENGINE_DECRYPTION_ENABLE: &str = "trt_engine_decryption_enable";
pub const TRT_ENGINE_DECRYPTION_LIB_PATH: &str = "trt_engine_decryption_lib_path";
pub const TRT_FORCE_SEQUENTIAL_ENGINE_BUILD: &str = "trt_force_sequential_engine_build";
pub const TRT_CONTEXT_MEMORY_SHARING_ENABLE: &str = "trt_context_memory_sharing_enable";
pub const TRT_LAYER_NORM_FP32_FALLBACK: &str = "trt_layer_norm_fp32_fallback";
pub const TRT_TIMING_CACHE_ENABLE: &str = "trt_timing_cache_enable";
pub const TRT_TIMING_CACHE_PATH: &str = "trt_timing_cache_path";
pub const TRT_FORCE_TIMING_CACHE: &str = "trt_force_timing_cache";
pub const TRT_DETAILED_BUILD_LOG: &str = "trt_detailed_build_log";
pub const TRT_BUILD_HEURISTICS_ENABLE: &str = "trt_build_heuristics_enable";
pub const TRT_SPARSITY_ENABLE: &str = "trt_sparsity_enable";
pub const TRT_BUILDER_OPTIMIZATION_LEVEL: &str = "trt_builder_optimization_level";
pub const TRT_AUXILIARY_STREAMS: &str = "trt_auxiliary_streams";
pub const TRT_TACTIC_SOURCES: &str = "trt_tactic_sources";
pub const TRT_EXTRA_PLUGIN_LIB_PATHS: &str = "trt_extra_plugin_lib_paths";
pub const TRT_PROFILE_MIN_SHAPES: &str = "trt_profile_min_shapes";
pub const TRT_PROFILE_MAX_SHAPES: &str = "trt_profile_max_shapes";
pub const TRT_PROFILE_OPT_SHAPES: &str = "trt_profile_opt_shapes";
pub const TRT_CUDA_GRAPH_ENABLE: &str = "trt_cuda_graph_enable";
pub const TRT_DUMP_SUBGRAPHS: &str = "trt_dump_subgraphs";
pub const TRT_ENGINE_HW_COMPATIBLE: &str = "trt_engine_hw_compatible";
pub const TRT_ONNX_MODEL_FOLDER_PATH: &str = "trt_onnx_model_folder_path";
pub const TRT_WEIGHT_STRIPPED_ENGINE_ENABLE: &str = "trt_weight_stripped_engine_enable";
pub const TRT_ENGINE_CACHE_PREFIX: &str = "trt_engine_cache_prefix";
pub const TRT_DUMP_EP_CONTEXT_MODEL: &str = "trt_dump_ep_context_model";
pub const TRT_EP_CONTEXT_FILE_PATH: &str = "trt_ep_context_file_path";
pub const TRT_EP_CONTEXT_EMBED_MODE: &str = "trt_ep_context_embed_mode";
pub const TRT_DLA_LOCAL_DRAM_SIZE: &str = "trt_dla_local_dram_size";
pub const TRT_DLA_GLOBAL_DRAM_SIZE: &str = "trt_dla_global_dram_size";
pub const TRT_MAX_BATCH_SIZE: &str = "trt_max_batch_size";
pub const TRT_DLA_SRAM_SIZE: &str = "trt_dla_sram_size";

// ============================================================================
// DirectML Execution Provider
// ============================================================================

pub const DISABLE_METACOMMANDS: &str = "disable_metacommands";
pub const ENABLE_DYNAMIC_GRAPH_FUSION: &str = "enable_dynamic_graph_fusion";
pub const ENABLE_CPU_SYNC_SPINNING: &str = "enable_cpu_sync_spinning";
pub const DISABLE_MEMORY_ARENA: &str = "disable_memory_arena";
pub const GRAPH_FUSION_FILTER_LEVEL: &str = "graph_fusion_filter_level";
pub const ENABLE_GPU_UPLOAD_HEAP: &str = "enable_gpu_upload_heap";
pub const ENABLE_METACOMMANDS: &str = "enable_metacommands";

// ============================================================================
// OpenVINO Execution Provider
// ============================================================================

pub const DEVICE_TYPE: &str = "device_type";
pub const NUM_OF_THREADS: &str = "num_of_threads";
pub const CACHE_DIR: &str = "cache_dir";
pub const ENABLE_OPENCL_THROTTLING: &str = "enable_opencl_throttling";
pub const ENABLE_QDQ_OPTIMIZER: &str = "enable_qdq_optimizer";
pub const DISABLE_DYNAMIC_SHAPES: &str = "disable_dynamic_shapes";
pub const NUM_STREAMS: &str = "num_streams";
pub const PRECISION: &str = "precision";
pub const ENABLE_NP_CACHED_DNNL_PRIMITIVE: &str = "enable_npu_cached_dnnl_primitive";
pub const ENABLE_MODEL_CACHING: &str = "enable_model_caching";
pub const ENABLE_NNCF: &str = "enable_nncf";
pub const ENABLE_DYNAMIC_SHAPES: &str = "enable_dynamic_shapes";
pub const EXECUTION_MODE: &str = "execution_mode";

// ============================================================================
// ROCm Execution Provider
// ============================================================================

pub const MIOPEN_CONV_ALGO_SEARCH: &str = "miopen_conv_algo_search";
pub const MIOPEN_CONV_USE_MAX_WORKSPACE: &str = "miopen_conv_use_max_workspace";

// ============================================================================
// CoreML Execution Provider
// ============================================================================

pub const COREML_FLAGS: &str = "CoreMLFlags";
pub const REQUIRE_STATIC_INPUT_SHAPES: &str = "RequireStaticInputShapes";
pub const ENABLE_ON_SUBGRAPHS: &str = "EnableOnSubgraphs";
pub const ONLY_ENABLE_DEVICE_WITH_ANE: &str = "OnlyEnableDeviceWithANE";
pub const ONLY_ALLOW_STATIC_INPUT_SHAPES: &str = "OnlyAllowStaticInputShapes";
pub const CREATE_ML_PROGRAM: &str = "CreateMLProgram";
pub const ALLOW_LOW_PRECISION_ACCUMULATION_ON_GPU: &str = "AllowLowPrecisionAccumulationOnGPU";
pub const ENABLE_MODEL_IO_NAME_CAPTURE: &str = "EnableModelIONameCapture";
pub const GET_SHAPE_STRATEGY: &str = "GetShapeStrategy";
pub const ML_MODEL_FORMAT: &str = "MLModelFormat";
pub const ML_COMPUTE_UNITS: &str = "MLComputeUnits";
pub const ENABLE_ON_SUBGRAPH: &str = "EnableOnSubgraph";
pub const MINIMUM_DEPLOYMENT_TARGET: &str = "MinimumDeploymentTarget";
pub const CREATE_ML_PROGRAM_IN_MEMORY: &str = "CreateMLProgramInMemory";
pub const MAX_WAIT_TIME_SECONDS: &str = "MaxWaitTimeSeconds";

// ============================================================================
// ACL Execution Provider
// ============================================================================

pub const FAST_MATH: &str = "fast_math";

// ============================================================================
// Azure Execution Provider
// ============================================================================

pub const ENDPOINT: &str = "endpoint";
pub const AUTH_TOKEN: &str = "auth_token";

// ============================================================================
// CANN Execution Provider
// ============================================================================

pub const NPU_MEM_LIMIT: &str = "npu_mem_limit";
pub const ENABLE_CANN_GRAPH: &str = "enable_cann_graph";
pub const DUMP_GRAPHS: &str = "dump_graphs";
pub const PRECISION_MODE: &str = "precision_mode";
pub const OP_SELECT_IMPL_MODE: &str = "op_select_impl_mode";

// ============================================================================
// MIGraphX Execution Provider
// ============================================================================

pub const MIGRAPHX_FP16_ENABLE: &str = "migraphx_fp16_enable";
pub const MIGRAPHX_INT8_ENABLE: &str = "migraphx_int8_enable";
pub const MIGRAPHX_INT8_CALIBRATION_TABLE_NAME: &str = "migraphx_int8_calibration_table_name";
pub const MIGRAPHX_USE_NATIVE_CALIBRATION_TABLE: &str = "migraphx_use_native_calibration_table";
pub const MIGRAPHX_SAVE_MODEL_PATH: &str = "migraphx_save_model_path";
pub const MIGRAPHX_SAVE_COMPILED_MODEL: &str = "migraphx_save_compiled_model";
pub const MIGRAPHX_LOAD_MODEL_PATH: &str = "migraphx_load_model_path";
pub const MIGRAPHX_LOAD_COMPILED_MODEL: &str = "migraphx_load_compiled_model";
pub const MIGRAPHX_EXHAUSTIVE_TUNE: &str = "migraphx_exhaustive_tune";

// ============================================================================
// NNAPI Execution Provider
// ============================================================================

pub const USE_FP16: &str = "use_fp16";
pub const USE_NCHW: &str = "use_nchw";
pub const DISABLE_CPU: &str = "disable_cpu";
pub const CPU_ONLY: &str = "cpu_only";

// ============================================================================
// NV Execution Provider
// ============================================================================

pub const EP_NV_DEVICE_ID: &str = "ep.nvtensorrtrtxexecutionprovider.device_id";
pub const EP_NV_CUDA_GRAPH_ENABLE: &str = "ep.nvtensorrtrtxexecutionprovider.nv_cuda_graph_enable";

// ============================================================================
// QNN Execution Provider
// ============================================================================

pub const BACKEND_PATH: &str = "backend_path";
pub const PROFILING_LEVEL: &str = "profiling_level";
pub const RPC_CONTROL_LATENCY: &str = "rpc_control_latency";
pub const VTCM_MB: &str = "vtcm_mb";
pub const HTP_PERFORMANCE_MODE: &str = "htp_performance_mode";
pub const ENABLE_HTP_FP16_PRECISION: &str = "enable_htp_fp16_precision";

// ============================================================================
// TVM Execution Provider
// ============================================================================

pub const EXECUTOR: &str = "executor";
pub const SO_FOLDER: &str = "so_folder";
pub const CHECK_HASH: &str = "check_hash";
pub const TARGET: &str = "target";
pub const TARGET_HOST: &str = "target_host";
pub const OPT_LEVEL: &str = "opt_level";
pub const FREEZE_WEIGHTS: &str = "freeze_weights";
pub const TUNING_FILE_PATH: &str = "tuning_file_path";

// ============================================================================
// Vitis AI Execution Provider
// ============================================================================

pub const CONFIG_FILE: &str = "config_file";
pub const CACHE_KEY: &str = "cache_key";

// ============================================================================
// WebGPU Execution Provider
// ============================================================================

pub const WEBGPU_PREFERRED_LAYOUT: &str = "ep.webgpuexecutionprovider.preferredLayout";
pub const WEBGPU_ENABLE_GRAPH_CAPTURE: &str = "ep.webgpuexecutionprovider.enableGraphCapture";
pub const WEBGPU_DEVICE_ID: &str = "ep.webgpuexecutionprovider.deviceId";
pub const WEBGPU_STORAGE_BUFFER_CACHE_MODE: &str = "ep.webgpuexecutionprovider.storageBufferCacheMode";
pub const WEBGPU_VALIDATION_MODE: &str = "ep.webgpuexecutionprovider.validationMode";

// ============================================================================
// WebNN Execution Provider
// ============================================================================

pub const DEVICE_TYPE_WEBNN: &str = "deviceType";
pub const POWER_PREFERENCE: &str = "powerPreference";
pub const NUM_THREADS: &str = "numThreads";

// ============================================================================
// XNNPACK Execution Provider
// ============================================================================

pub const INTRA_OP_NUM_THREADS: &str = "intra_op_num_threads";

