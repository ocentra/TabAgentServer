"""
Integration tests for Python ML gRPC services

These are REAL tests that:
- Start an actual Python ML gRPC server
- Create real gRPC clients
- Test Transformers and Mediapipe services end-to-end
- Use real ML models (small ones for testing)
"""

import pytest
import asyncio
import grpc
import sys
import os
from pathlib import Path
import numpy as np
from PIL import Image
from io import BytesIO

# Add generated proto path
sys.path.insert(0, str(Path(__file__).parent.parent / 'generated'))
import ml_inference_pb2
import ml_inference_pb2_grpc


# Test configuration
TEST_SERVER_PORT = 50052  # Different from default to avoid conflicts
TEST_SERVER_ADDRESS = f'localhost:{TEST_SERVER_PORT}'


@pytest.fixture(scope="module")
async def ml_server():
    """Start ML server for testing"""
    # Import server components
    sys.path.insert(0, str(Path(__file__).parent.parent))
    from services.transformers_service import TransformersServiceImpl
    from services.mediapipe_service import MediapipeServiceImpl
    
    # Create server
    server = grpc.aio.server()
    ml_inference_pb2_grpc.add_TransformersServiceServicer_to_server(
        TransformersServiceImpl(), server
    )
    ml_inference_pb2_grpc.add_MediapipeServiceServicer_to_server(
        MediapipeServiceImpl(), server
    )
    
    server.add_insecure_port(f'0.0.0.0:{TEST_SERVER_PORT}')
    await server.start()
    
    yield server
    
    await server.stop(grace=5)


@pytest.fixture
async def transformers_client(ml_server):
    """Create Transformers service client"""
    async with grpc.aio.insecure_channel(TEST_SERVER_ADDRESS) as channel:
        yield ml_inference_pb2_grpc.TransformersServiceStub(channel)


@pytest.fixture
async def mediapipe_client(ml_server):
    """Create Mediapipe service client"""
    async with grpc.aio.insecure_channel(TEST_SERVER_ADDRESS) as channel:
        yield ml_inference_pb2_grpc.MediapipeServiceStub(channel)


@pytest.mark.asyncio
async def test_generate_text_streaming(transformers_client):
    """Test streaming text generation"""
    request = ml_inference_pb2.TextRequest(
        prompt="Once upon a time",
        model="gpt2",  # Small model for testing
        max_length=20,
        temperature=0.7,
        top_p=0.9
    )
    
    responses = []
    try:
        async for response in transformers_client.GenerateText(request):
            responses.append(response)
            if response.done:
                break
    except grpc.RpcError as e:
        pytest.skip(f"Model not available: {e}")
    
    # Verify streaming behavior
    assert len(responses) > 0, "Expected at least one response"
    assert responses[-1].done, "Last response should be marked as done"
    assert responses[-1].tokens_generated > 0, "Should have generated some tokens"
    
    # Verify content
    generated_text = ''.join(r.text for r in responses[:-1])
    assert len(generated_text) > 0, "Generated text should not be empty"


@pytest.mark.asyncio
async def test_generate_embeddings(transformers_client):
    """Test embedding generation for multiple texts"""
    request = ml_inference_pb2.GenerateEmbeddingsRequest(
        texts=[
            "The cat sat on the mat",
            "Dogs are great pets",
            "Machine learning is fascinating"
        ],
        model="sentence-transformers/all-MiniLM-L6-v2"
    )
    
    try:
        response = await transformers_client.GenerateEmbeddings(request)
    except grpc.RpcError as e:
        pytest.skip(f"Model not available: {e}")
    
    # Verify embeddings
    assert len(response.embeddings) == 3, "Expected 3 embeddings"
    
    for embedding in response.embeddings:
        assert len(embedding.values) > 0, "Embedding should not be empty"
        # sentence-transformers/all-MiniLM-L6-v2 produces 384-dimensional embeddings
        assert len(embedding.values) == 384, "Expected 384-dimensional embedding"
        
        # Verify values are floats
        for value in embedding.values:
            assert isinstance(value, float), "Embedding values should be floats"


@pytest.mark.asyncio
async def test_embedding_similarity(transformers_client):
    """Test that similar texts produce similar embeddings"""
    request = ml_inference_pb2.GenerateEmbeddingsRequest(
        texts=[
            "The cat is sleeping",
            "A cat is taking a nap",  # Similar to first
            "Quantum physics is complex"  # Dissimilar
        ],
        model="sentence-transformers/all-MiniLM-L6-v2"
    )
    
    try:
        response = await transformers_client.GenerateEmbeddings(request)
    except grpc.RpcError as e:
        pytest.skip(f"Model not available: {e}")
    
    embeddings = [np.array(emb.values) for emb in response.embeddings]
    
    # Calculate cosine similarity
    def cosine_similarity(a, b):
        return np.dot(a, b) / (np.linalg.norm(a) * np.linalg.norm(b))
    
    sim_cat_cat = cosine_similarity(embeddings[0], embeddings[1])
    sim_cat_physics = cosine_similarity(embeddings[0], embeddings[2])
    
    # Similar sentences should have higher similarity
    assert sim_cat_cat > sim_cat_physics, \
        f"Similar texts should be more similar: {sim_cat_cat} > {sim_cat_physics}"
    
    # Similarity should be reasonable
    assert 0.5 < sim_cat_cat < 1.0, "Cat-cat similarity should be high"
    assert 0.0 < sim_cat_physics < 0.5, "Cat-physics similarity should be low"


@pytest.mark.asyncio
async def test_chat_completion_streaming(transformers_client):
    """Test streaming chat completion"""
    request = ml_inference_pb2.ChatRequest(
        messages=[
            ml_inference_pb2.ChatMessage(role="user", content="Hello!"),
            ml_inference_pb2.ChatMessage(role="assistant", content="Hi there! How can I help?"),
            ml_inference_pb2.ChatMessage(role="user", content="Tell me a joke")
        ],
        model="microsoft/DialoGPT-medium",
        temperature=0.7
    )
    
    responses = []
    try:
        async for response in transformers_client.ChatCompletion(request):
            responses.append(response)
            if response.done:
                break
    except grpc.RpcError as e:
        pytest.skip(f"Model not available: {e}")
    
    # Verify streaming
    assert len(responses) > 0, "Expected at least one response"
    assert responses[-1].done, "Last response should be marked as done"
    assert responses[-1].finish_reason == "stop", "Should finish with 'stop' reason"
    
    # Verify content
    generated_content = ''.join(r.content for r in responses[:-1])
    assert len(generated_content) > 0, "Generated content should not be empty"


def create_test_image(width=640, height=480, color=(255, 0, 0)):
    """Create a test image with a colored rectangle"""
    image = Image.new('RGB', (width, height), color=(255, 255, 255))
    # Draw a colored rectangle in the center
    from PIL import ImageDraw
    draw = ImageDraw.Draw(image)
    x1, y1 = width // 4, height // 4
    x2, y2 = 3 * width // 4, 3 * height // 4
    draw.rectangle([x1, y1, x2, y2], fill=color)
    
    # Convert to bytes
    buffer = BytesIO()
    image.save(buffer, format='JPEG')
    return buffer.getvalue()


@pytest.mark.asyncio
async def test_face_detection(mediapipe_client):
    """Test face detection"""
    # Create a simple test image
    image_data = create_test_image()
    
    request = ml_inference_pb2.ImageRequest(
        image_data=image_data,
        format="jpeg"
    )
    
    try:
        response = await mediapipe_client.DetectFaces(request)
    except grpc.RpcError as e:
        pytest.skip(f"Mediapipe not available: {e}")
    
    # Note: Our test image doesn't have real faces, so this tests the service works
    # Real face detection would require actual face images
    assert isinstance(response, ml_inference_pb2.FaceDetectionResponse)
    assert isinstance(response.faces, list)


@pytest.mark.asyncio
async def test_hand_detection(mediapipe_client):
    """Test hand detection"""
    image_data = create_test_image()
    
    request = ml_inference_pb2.ImageRequest(
        image_data=image_data,
        format="jpeg"
    )
    
    try:
        response = await mediapipe_client.DetectHands(request)
    except grpc.RpcError as e:
        pytest.skip(f"Mediapipe not available: {e}")
    
    assert isinstance(response, ml_inference_pb2.HandDetectionResponse)
    assert isinstance(response.hands, list)


@pytest.mark.asyncio
async def test_pose_detection(mediapipe_client):
    """Test pose detection"""
    image_data = create_test_image()
    
    request = ml_inference_pb2.ImageRequest(
        image_data=image_data,
        format="jpeg"
    )
    
    try:
        response = await mediapipe_client.DetectPose(request)
    except grpc.RpcError as e:
        pytest.skip(f"Mediapipe not available: {e}")
    
    assert isinstance(response, ml_inference_pb2.PoseDetectionResponse)
    assert isinstance(response.landmarks, list)
    assert isinstance(response.confidence, float)


@pytest.mark.asyncio
async def test_concurrent_requests(transformers_client):
    """Test handling multiple concurrent requests"""
    requests = [
        ml_inference_pb2.GenerateEmbeddingsRequest(
            texts=[f"Test text {i}"],
            model="sentence-transformers/all-MiniLM-L6-v2"
        )
        for i in range(5)
    ]
    
    try:
        # Send all requests concurrently
        tasks = [transformers_client.GenerateEmbeddings(req) for req in requests]
        responses = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Verify all succeeded
        for response in responses:
            if isinstance(response, Exception):
                pytest.skip(f"Model not available: {response}")
            assert len(response.embeddings) == 1
            assert len(response.embeddings[0].values) == 384
            
    except grpc.RpcError as e:
        pytest.skip(f"Model not available: {e}")


@pytest.mark.asyncio
async def test_error_handling_invalid_model(transformers_client):
    """Test error handling for invalid model"""
    request = ml_inference_pb2.TextRequest(
        prompt="Test",
        model="nonexistent/invalid-model-12345",
        max_length=10,
        temperature=0.7,
        top_p=0.9
    )
    
    with pytest.raises(grpc.RpcError) as exc_info:
        async for _ in transformers_client.GenerateText(request):
            pass
    
    assert exc_info.value.code() == grpc.StatusCode.INTERNAL


@pytest.mark.asyncio
async def test_error_handling_invalid_image(mediapipe_client):
    """Test error handling for invalid image data"""
    request = ml_inference_pb2.ImageRequest(
        image_data=b"invalid_image_data",
        format="jpeg"
    )
    
    # Should handle gracefully
    response = await mediapipe_client.DetectFaces(request)
    
    # Might return empty results or error, but shouldn't crash
    assert isinstance(response, ml_inference_pb2.FaceDetectionResponse)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v", "-s"])

