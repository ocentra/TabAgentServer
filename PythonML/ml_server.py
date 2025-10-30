#!/usr/bin/env python3
"""
TabAgent Python ML gRPC Server

Provides ML inference services via gRPC:
- ModelManagementService (load/unload models, file serving)
- TransformersService (text generation, embeddings, chat)
- MediapipeService (face/hand/pose detection)
"""

import argparse
import asyncio
import logging
from concurrent import futures
import grpc

# Add PythonML root to sys.path for local imports
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from services.model_management_service import ModelManagementServiceImpl
from services.transformers_service import TransformersServiceImpl
from services.mediapipe_service import MediapipeService
from generated import ml_inference_pb2_grpc


logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


async def serve(port: int):
    """Start the gRPC server"""
    server = grpc.aio.server(
        futures.ThreadPoolExecutor(max_workers=10),
        options=[
            ('grpc.max_send_message_length', 50 * 1024 * 1024),  # 50MB
            ('grpc.max_receive_message_length', 50 * 1024 * 1024),
        ]
    )
    
    # Create a placeholder gRPC channel for Rust file provider
    # This will be used by ModelManagementService to fetch files from Rust
    # For now, we'll use localhost:50052 (assuming Rust serves on a different port)
    # TODO: Make this configurable via CLI args
    rust_grpc_channel = grpc.aio.insecure_channel('localhost:50052')
    
    # Initialize services
    model_mgmt_service = ModelManagementServiceImpl()
    transformers_service = TransformersServiceImpl(model_mgmt_service)
    mediapipe_service = MediapipeService()
    
    # Register services
    ml_inference_pb2_grpc.add_ModelManagementServiceServicer_to_server(
        model_mgmt_service, server
    )
    ml_inference_pb2_grpc.add_TransformersServiceServicer_to_server(
        transformers_service, server
    )
    ml_inference_pb2_grpc.add_MediapipeServiceServicer_to_server(
        mediapipe_service, server
    )
    
    listen_addr = f'0.0.0.0:{port}'
    server.add_insecure_port(listen_addr)
    
    logger.info('=' * 80)
    logger.info(f'🚀 Starting Python ML gRPC Server on {listen_addr}')
    logger.info('=' * 80)
    logger.info('📦 Services available:')
    logger.info('  ✅ ModelManagementService - Load/unload models, manage lifecycle')
    logger.info('  ✅ TransformersService - Text generation, embeddings, chat')
    logger.info('  ✅ MediapipeService - ALL STREAMING:')
    logger.info('      • Face detection/mesh/iris (real-time)')
    logger.info('      • Hand tracking/gestures (real-time)')
    logger.info('      • Pose tracking (real-time)')
    logger.info('      • Holistic tracking (face+hands+pose)')
    logger.info('      • Segmentation (selfie/hair)')
    logger.info('      • Object detection/tracking/3D')
    logger.info('      • Template matching, AutoFlip, Media sequence')
    logger.info('=' * 80)
    
    await server.start()
    logger.info('✅ Python ML gRPC server started successfully')
    logger.info('💡 Waiting for Rust to send model loading requests...')
    
    try:
        await server.wait_for_termination()
    except KeyboardInterrupt:
        logger.info('\n🛑 Shutting down Python ML gRPC server...')
        await server.stop(grace=5)
        logger.info('✅ Server stopped gracefully')


def main():
    parser = argparse.ArgumentParser(description='TabAgent Python ML gRPC Server')
    parser.add_argument(
        '--port',
        type=int,
        default=50051,
        help='Port to listen on (default: 50051)'
    )
    parser.add_argument(
        '--rust-port',
        type=int,
        default=50052,
        help='Port where Rust gRPC server is running (default: 50052)'
    )
    args = parser.parse_args()
    
    try:
        asyncio.run(serve(args.port))
    except Exception as e:
        logger.error(f'❌ Server error: {e}', exc_info=True)
        sys.exit(1)


if __name__ == '__main__':
    main()
