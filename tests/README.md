# TabAgent Server Tests

Comprehensive test suite for API, backends, and integrations.

## Test Structure

```
tests/
├── conftest.py                    # Pytest fixtures and configuration
├── test_api_health.py            # Health endpoint tests
├── test_api_models.py            # Models endpoint tests
├── test_api_chat.py              # Chat completions tests
├── test_backend_adapter.py       # Backend adapter tests
├── test_backend_manager.py       # Backend manager tests
├── test_integration_api.py       # Integration tests
├── test_types.py                 # Pydantic model tests
└── test_host.py                  # Native messaging tests (existing)
```

## Running Tests

### Install Test Dependencies

```bash
pip install pytest pytest-asyncio pytest-cov httpx
```

### Run All Tests

```bash
# From Server/ directory
pytest tests/ -v
```

### Run Specific Test File

```bash
pytest tests/test_api_health.py -v
```

### Run with Coverage

```bash
pytest tests/ --cov=api --cov=backends --cov-report=html
```

### Run Only Unit Tests

```bash
pytest tests/ -v -k "not integration"
```

### Run Only Integration Tests

```bash
pytest tests/ -v -k "integration"
```

## Test Categories

### Unit Tests
- `test_api_*.py` - Individual endpoint tests
- `test_backend_*.py` - Backend component tests
- `test_types.py` - Type validation tests

### Integration Tests
- `test_integration_api.py` - Full API workflow tests

### Native Messaging Tests
- `test_host.py` - Native host communication tests

## Writing New Tests

### Follow These Rules

1. **No String Literals** - Use constants from `api.constants`
2. **Strong Typing** - Type hints on all functions
3. **Use Fixtures** - Leverage conftest.py fixtures
4. **Clear Naming** - Test names describe what they test
5. **AAA Pattern** - Arrange, Act, Assert

### Example Test

```python
from api.constants import APIPrefix, EndpointPath

class TestMyFeature:
    """Tests for my feature"""
    
    def test_feature_works(self, client: TestClient):
        """Test feature works correctly"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.HEALTH.value}"
        
        # Act
        response = client.get(endpoint)
        
        # Assert
        assert response.status_code == 200
```

## Fixtures Available

- `client` - FastAPI TestClient
- `mock_backend_manager` - Mocked backend for testing
- `sample_chat_messages` - Sample ChatMessage list
- `sample_inference_settings` - Sample InferenceSettings

## Coverage Goals

- **API Routes**: 90%+ coverage
- **Backend Adapters**: 85%+ coverage
- **Types**: 100% coverage (simple validation)
- **Integration**: Key workflows covered

