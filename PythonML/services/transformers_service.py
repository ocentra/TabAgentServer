"""
Transformers Service Implementation

Provides text generation, embeddings, and chat completion using the pipeline system.
Delegates to ModelManagementService for loading/unloading models.
"""

import logging
import grpc

# Add PythonML root to sys.path for local imports
import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from generated import ml_inference_pb2
from generated import ml_inference_pb2_grpc

logger = logging.getLogger(__name__)


class TransformersService(ml_inference_pb2_grpc.TransformersServiceServicer):
    """
    Implementation of the Transformers gRPC service.
    
    Uses the pipeline system for inference. Models are managed by ModelManagementService.
    """
    
    def __init__(self, model_management_service):
        """
        Initialize the transformers service.
        
        Args:
            model_management_service: Reference to ModelManagementService for accessing loaded pipelines
        """
        self.model_mgmt = model_management_service
        logger.info("TransformersService initialized")
    
    def _get_pipeline(self, model_id: str):
        """Get a loaded pipeline from ModelManagementService"""
        return self.model_mgmt.loaded_models.get(model_id)
    
    async def GenerateText(self, request, context):
        """
        Stream text generation token by token.
        
        Note: Currently returns complete generation due to streaming complexity.
        TODO: Implement proper token-by-token streaming with transformers TextIteratorStreamer.
        """
        try:
            model_id = request.model
            if not model_id:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("model field is required")
                return
            
            # Check if model is loaded
            pipeline = self._get_pipeline(model_id)
            if not pipeline:
                context.set_code(grpc.StatusCode.FAILED_PRECONDITION)
                context.set_details(f"Model {model_id} not loaded. Load it first using ModelManagementService.")
                return
            
            # Prepare generation input
            input_data = {
                "prompt": request.prompt,
                "max_new_tokens": request.max_length if request.max_length > 0 else 100,
                "temperature": request.temperature if request.temperature > 0 else 0.7,
                "top_p": request.top_p if request.top_p > 0 else 0.9,
                "do_sample": True
            }
            
            # Generate (currently non-streaming)
            result = pipeline.generate(input_data)
            
            if result.get("status") == "error":
                context.set_code(grpc.StatusCode.INTERNAL)
                context.set_details(f"Generation failed: {result.get('message')}")
                return
            
            generated_text = result.get("text", "")
            tokens_generated = result.get("tokens_generated", 0)
            
            # Yield complete response (TODO: implement streaming)
            yield ml_inference_pb2.TextResponse(
                text=generated_text,
                done=True,
                tokens_generated=tokens_generated
            )
            
        except Exception as e:
            logger.error(f"Error in GenerateText: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(f"Internal error: {str(e)}")
            return
    
    async def GenerateEmbeddings(self, request, context):
        """Generate embeddings for multiple texts"""
        try:
            model_id = request.model
            if not model_id:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("model field is required")
                return ml_inference_pb2.GeneratedEmbeddingsResponse()
            
            if not request.texts:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("texts field is required")
                return ml_inference_pb2.GeneratedEmbeddingsResponse()
            
            # Check if model is loaded
            pipeline = self._get_pipeline(model_id)
            if not pipeline:
                context.set_code(grpc.StatusCode.FAILED_PRECONDITION)
                context.set_details(f"Model {model_id} not loaded. Load it first using ModelManagementService.")
                return ml_inference_pb2.GeneratedEmbeddingsResponse()
            
            # Prepare input
            input_data = {
                "texts": list(request.texts),
                "normalize_embeddings": True,
                "convert_to_numpy": False
            }
            
            # Generate embeddings
            result = pipeline.generate(input_data)
            
            if result.get("status") == "error":
                context.set_code(grpc.StatusCode.INTERNAL)
                context.set_details(f"Embedding generation failed: {result.get('message')}")
                return ml_inference_pb2.GeneratedEmbeddingsResponse()
            
            # Convert to protobuf format
            embeddings_data = result.get("embeddings", [])
            
            # Handle both single embedding and multiple embeddings
            if isinstance(embeddings_data, list) and embeddings_data:
                # Check if it's a list of embeddings or a single embedding
                if isinstance(embeddings_data[0], (list, tuple)):
                    # Multiple embeddings
                    embeddings = [
                        ml_inference_pb2.GeneratedEmbedding(values=emb)
                        for emb in embeddings_data
                    ]
                else:
                    # Single embedding (was returned as flat list)
                    embeddings = [ml_inference_pb2.GeneratedEmbedding(values=embeddings_data)]
            else:
                embeddings = []
            
            return ml_inference_pb2.GeneratedEmbeddingsResponse(embeddings=embeddings)
            
        except Exception as e:
            logger.error(f"Error in GenerateEmbeddings: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(f"Internal error: {str(e)}")
            return ml_inference_pb2.GeneratedEmbeddingsResponse()
    
    async def ChatCompletion(self, request, context):
        """
        Stream chat completion responses.
        
        Note: Currently returns complete response due to streaming complexity.
        TODO: Implement proper token-by-token streaming.
        """
        try:
            model_id = request.model
            if not model_id:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("model field is required")
                return
            
            if not request.messages:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("messages field is required")
                return
            
            # Check if model is loaded
            pipeline = self._get_pipeline(model_id)
            if not pipeline:
                context.set_code(grpc.StatusCode.FAILED_PRECONDITION)
                context.set_details(f"Model {model_id} not loaded. Load it first using ModelManagementService.")
                return
            
            # Format chat messages as a prompt
            # TODO: Use chat template from tokenizer if available
            prompt = ""
            for msg in request.messages:
                role = msg.role.capitalize()
                prompt += f"{role}: {msg.content}\n"
            prompt += "Assistant: "
            
            # Prepare generation input
            input_data = {
                "prompt": prompt,
                "max_new_tokens": 512,
                "temperature": request.temperature if request.temperature > 0 else 0.7,
                "do_sample": True
            }
            
            # Generate
            result = pipeline.generate(input_data)
            
            if result.get("status") == "error":
                context.set_code(grpc.StatusCode.INTERNAL)
                context.set_details(f"Chat completion failed: {result.get('message')}")
                return
            
            generated_text = result.get("text", "")
            
            # Yield complete response (TODO: implement streaming)
            yield ml_inference_pb2.ChatResponse(
                content=generated_text,
                done=True,
                finish_reason="stop"
            )
            
        except Exception as e:
            logger.error(f"Error in ChatCompletion: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(f"Internal error: {str(e)}")
            return
