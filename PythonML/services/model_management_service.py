"""
Model Management Service
========================

Handles model lifecycle: load, unload, file serving.
Uses the pipeline system to support all model types (Florence2, Whisper, CLIP, etc.)

This is the bridge between Rust's orchestration and Python's inference.
"""

import logging
import psutil
import sys
from pathlib import Path
from typing import Dict, Optional
from datetime import datetime

# Add parent dirs to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from generated import ml_inference_pb2, ml_inference_pb2_grpc
    from pipelines import PipelineFactory
    from pipelines.base import BasePipeline
    from core.rust_file_provider import RustFileProvider
except ImportError as e:
    logging.error(f"Failed to import dependencies: {e}")
    ml_inference_pb2 = None
    ml_inference_pb2_grpc = None
    PipelineFactory = None

logger = logging.getLogger(__name__)


class ModelManagementService(ml_inference_pb2_grpc.ModelManagementServiceServicer):
    """
    gRPC service for model lifecycle management.
    
    Rust calls this to:
    - Load models with specific pipelines
    - Unload models to free memory
    - Serve model files from Rust's cache
    """
    
    def __init__(self):
        self.loaded_models: Dict[str, BasePipeline] = {}
        self.model_metadata: Dict[str, dict] = {}
        self.file_provider: Optional[RustFileProvider] = None
        logger.info("ModelManagementService initialized")
    
    def set_file_provider(self, provider: RustFileProvider):
        """Set the Rust file provider (called after gRPC connection established)"""
        self.file_provider = provider
        logger.info("Rust file provider connected")
    
    async def LoadModel(self, request, context):
        """
        Load a model into memory with specified pipeline.
        
        Rust decides:
        - Which model ID
        - Which pipeline type (florence2, whisper, text-generation, etc.)
        - Optional architecture hint
        - Loading options
        
        Python:
        - Uses PipelineFactory to create correct pipeline
        - Loads model using Rust's file provider
        - Returns memory usage
        - FAILS HARD on errors (Rust handles retry/fallback)
        """
        model_id = request.model_id
        pipeline_type = request.pipeline_type
        architecture = request.architecture or None
        
        logger.info(f"📥 LoadModel request: model_id={model_id}, pipeline={pipeline_type}, arch={architecture}")
        
        try:
            # Check if already loaded
            if model_id in self.loaded_models:
                logger.warning(f"Model {model_id} already loaded, returning success")
                return ml_inference_pb2.LoadModelResponse(
                    success=True,
                    message=f"Model {model_id} already loaded",
                    vram_allocated_mb=0,
                    ram_allocated_mb=0
                )
            
            # Ensure we have file provider
            if not self.file_provider:
                raise RuntimeError("RustFileProvider not initialized")
            
            # Get RAM before loading
            process = psutil.Process()
            ram_before = process.memory_info().rss / (1024 * 1024)  # MB
            
            # Create pipeline using factory
            logger.info(f"Creating pipeline: type={pipeline_type}, arch={architecture}")
            pipeline = PipelineFactory.create_pipeline(
                task=pipeline_type,
                model_id=model_id,
                architecture=architecture
            )
            
            if not pipeline:
                raise RuntimeError(f"PipelineFactory failed to create pipeline for {pipeline_type}")
            
            logger.info(f"Pipeline created: {pipeline.__class__.__name__}")
            
            # Configure pipeline to use Rust's file provider
            # This ensures ALL file requests go through Rust's ModelCache
            pipeline.file_provider = self.file_provider
            
            # Load the model
            logger.info(f"Loading model {model_id}...")
            load_result = pipeline.load(
                model_id=model_id,
                options=dict(request.options) if request.options else {}
            )
            
            if load_result.get("status") == "error":
                raise RuntimeError(f"Pipeline load failed: {load_result.get('message', 'Unknown error')}")
            
            # Calculate memory usage
            ram_after = process.memory_info().rss / (1024 * 1024)  # MB
            ram_allocated = max(0, int(ram_after - ram_before))
            
            # Store loaded model
            self.loaded_models[model_id] = pipeline
            self.model_metadata[model_id] = {
                "pipeline_type": pipeline_type,
                "architecture": architecture,
                "loaded_at": datetime.now().timestamp(),
                "ram_mb": ram_allocated,
                "vram_mb": 0  # TODO: Get actual VRAM usage if GPU available
            }
            
            logger.info(f"✅ Model {model_id} loaded successfully (RAM: {ram_allocated}MB)")
            
            return ml_inference_pb2.LoadModelResponse(
                success=True,
                message=f"Model {model_id} loaded with {pipeline.__class__.__name__}",
                vram_allocated_mb=0,  # TODO: Implement VRAM tracking
                ram_allocated_mb=ram_allocated
            )
            
        except Exception as e:
            error_msg = f"Failed to load model {model_id}: {str(e)}"
            logger.error(f"❌ {error_msg}", exc_info=True)
            
            # Fail hard - Rust decides what to do
            context.set_code(context.StatusCode.INTERNAL)
            context.set_details(error_msg)
            return ml_inference_pb2.LoadModelResponse(
                success=False,
                message=error_msg,
                vram_allocated_mb=0,
                ram_allocated_mb=0
            )
    
    async def UnloadModel(self, request, context):
        """
        Unload a model from memory.
        
        Rust calls this when:
        - Switching models
        - Memory pressure
        - Explicit unload request
        """
        model_id = request.model_id
        logger.info(f"🗑️  UnloadModel request: {model_id}")
        
        try:
            if model_id not in self.loaded_models:
                logger.warning(f"Model {model_id} not loaded, nothing to unload")
                return ml_inference_pb2.StatusResponse(
                    success=True,
                    message=f"Model {model_id} was not loaded"
                )
            
            # Get pipeline and unload
            pipeline = self.loaded_models[model_id]
            pipeline.unload()
            
            # Remove from tracking
            del self.loaded_models[model_id]
            del self.model_metadata[model_id]
            
            logger.info(f"✅ Model {model_id} unloaded")
            
            return ml_inference_pb2.StatusResponse(
                success=True,
                message=f"Model {model_id} unloaded successfully"
            )
            
        except Exception as e:
            error_msg = f"Failed to unload model {model_id}: {str(e)}"
            logger.error(f"❌ {error_msg}", exc_info=True)
            
            # Fail hard
            context.set_code(context.StatusCode.INTERNAL)
            context.set_details(error_msg)
            return ml_inference_pb2.StatusResponse(
                success=False,
                message=error_msg
            )
    
    async def GetModelFile(self, request, context):
        """
        Serve a model file from Rust's cache (streamed).
        
        Python needs a file (config.json, model weights, etc.) →
        Asks Rust for it via this RPC →
        Rust serves from ModelCache.
        """
        model_id = request.model_id
        file_path = request.file_path
        
        logger.info(f"📂 GetModelFile request: {model_id}/{file_path}")
        
        try:
            if not self.file_provider:
                raise RuntimeError("RustFileProvider not initialized")
            
            # Get file from Rust
            data = self.file_provider.get_file(model_id, file_path)
            
            # Stream back in chunks (100KB chunks)
            CHUNK_SIZE = 100 * 1024
            total_size = len(data)
            offset = 0
            
            while offset < total_size:
                chunk_data = data[offset:offset + CHUNK_SIZE]
                is_last = (offset + len(chunk_data)) >= total_size
                
                yield ml_inference_pb2.ModelFileChunk(
                    data=chunk_data,
                    offset=offset,
                    total_size=total_size,
                    is_last=is_last
                )
                
                offset += len(chunk_data)
            
            logger.info(f"✅ Served {file_path} ({total_size} bytes)")
            
        except Exception as e:
            error_msg = f"Failed to get file {file_path}: {str(e)}"
            logger.error(f"❌ {error_msg}", exc_info=True)
            
            # Fail hard
            context.set_code(context.StatusCode.INTERNAL)
            context.set_details(error_msg)
    
    async def GetLoadedModels(self, request, context):
        """Get list of currently loaded models"""
        logger.info("📋 GetLoadedModels request")
        
        try:
            models = []
            for model_id, metadata in self.model_metadata.items():
                models.append(ml_inference_pb2.LoadedModelInfo(
                    model_id=model_id,
                    pipeline_type=metadata["pipeline_type"],
                    vram_mb=metadata.get("vram_mb", 0),
                    ram_mb=metadata.get("ram_mb", 0),
                    loaded_at=int(metadata["loaded_at"])
                ))
            
            logger.info(f"✅ Returning {len(models)} loaded models")
            
            return ml_inference_pb2.LoadedModelsResponse(models=models)
            
        except Exception as e:
            error_msg = f"Failed to get loaded models: {str(e)}"
            logger.error(f"❌ {error_msg}", exc_info=True)
            
            context.set_code(context.StatusCode.INTERNAL)
            context.set_details(error_msg)
            return ml_inference_pb2.LoadedModelsResponse(models=[])
    
    def get_pipeline(self, model_id: str) -> Optional[BasePipeline]:
        """Get a loaded pipeline (used by TransformersService)"""
        return self.loaded_models.get(model_id)

