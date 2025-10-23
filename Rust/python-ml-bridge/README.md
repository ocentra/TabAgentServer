# ML Bridge - Python ML Inference via PyO3

**Phase 6 Complete** | Rust-Python integration for ML model inference.

## Purpose

The ML Bridge provides a clean boundary between Rust's high-performance core and Python's rich ML ecosystem. It implements the `MlBridge` trait using PyO3 to call Python ML functions.

## Architecture

```
Rust (Weaver)
  ├─▶ MlBridge trait
  │     └─▶ PyMlBridge (this crate)
  │           └─▶ PyO3 FFI
  │                 └─▶ Python ml_funcs.py
  │                       ├─▶ sentence-transformers (embeddings)
  │                       ├─▶ spaCy (NER)
  │                       └─▶ transformers (summarization)
```

## Components

### Rust Side (`src/lib.rs`)

```rust
pub struct PyMlBridge {
    module_path: String,
}

impl PyMlBridge {
    pub fn new(python_module_path: impl AsRef<Path>) -> Result<Self, MlBridgeError>;
}

#[async_trait]
impl MlBridge for PyMlBridge {
    async fn generate_embedding(&self, text: &str) -> DbResult<Vec<f32>>;
    async fn extract_entities(&self, text: &str) -> DbResult<Vec<Entity>>;
    async fn summarize(&self, messages: &[String]) -> DbResult<String>;
    async fn health_check(&self) -> DbResult<bool>;
}
```

### Python Side (`python/ml_funcs.py`)

```python
def generate_embedding(text: str) -> list[float]:
    """Generate 384-dim embedding using all-MiniLM-L6-v2"""
    
def extract_entities(text: str) -> list[dict]:
    """Extract named entities using spaCy en_core_web_sm"""
    
def summarize(messages: list[str]) -> str:
    """Summarize using facebook/bart-large-cnn"""
```

## Setup

### 1. Install Python Dependencies

```bash
cd ml-bridge/python
pip install -r requirements.txt
python -m spacy download en_core_web_sm
```

### 2. Build with PyO3

```bash
# Windows
$env:PYO3_USE_ABI3_FORWARD_COMPATIBILITY="1"
cargo build --release -p ml-bridge

# Linux/Mac
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build --release -p ml-bridge
```

## Usage

```rust
use ml_bridge::PyMlBridge;
use weaver::ml_bridge::MlBridge;

// Initialize bridge
let bridge = PyMlBridge::new("path/to/ml-bridge/python")?;

// Generate embedding
let embedding = bridge.generate_embedding("Hello world").await?;
assert_eq!(embedding.len(), 384);

// Extract entities
let entities = bridge.extract_entities("Alice met Bob in Paris").await?;
// Returns: [
//   {text: "Alice", label: "PERSON", start: 0, end: 5},
//   {text: "Bob", label: "PERSON", start: 10, end: 13},
//   {text: "Paris", label: "GPE", start: 17, end: 22}
// ]
```

## Python Models

| Function | Model | Size | Output |
|----------|-------|------|--------|
| `generate_embedding` | sentence-transformers/all-MiniLM-L6-v2 | ~80MB | 384-dim vector |
| `extract_entities` | spaCy en_core_web_sm | ~15MB | List of entities |
| `summarize` | facebook/bart-large-cnn | ~1.6GB | Summary string |

## Performance

- **Embedding**: ~20-50ms per text (depends on length)
- **Entity Extraction**: ~50-100ms per text
- **Summarization**: ~200-500ms per batch

Models are loaded once and cached in memory for fast inference.

## Error Handling

```rust
pub enum MlBridgeError {
    PythonInit(String),      // Python initialization failed
    ModuleImport(String),    // Module not found
    FunctionCall(String),    // Python function error
    TypeConversion(String),  // Return value conversion error
}

impl From<MlBridgeError> for common::DbError {
    fn from(err: MlBridgeError) -> Self {
        common::DbError::Other(err.to_string())
    }
}
```

## Testing

```bash
# Rust tests (use MockMlBridge)
cargo test -p ml-bridge

# Python tests
python ml-bridge/python/ml_funcs.py
```

✅ 3 tests passing

## Development

### Testing Without Python

Use the `MockMlBridge` from weaver for development:

```rust
use weaver::ml_bridge::MockMlBridge;

let bridge = MockMlBridge;
let embedding = bridge.generate_embedding("test").await?;
// Returns mock 384-dim vector
```

### Swapping ML Backends

The `MlBridge` trait is backend-agnostic:
- Use `PyMlBridge` for Python models
- Use `MockMlBridge` for testing
- Future: Implement for ONNX Runtime, OpenAI API, etc.

## Dependencies

**Rust:**
- `pyo3` 0.20 - Python bindings
- `tokio` - Async runtime
- `async-trait` - Async trait support

**Python:**
- `sentence-transformers` >= 2.2.0
- `spacy` >= 3.7.0
- `transformers` >= 4.35.0
- `torch` >= 2.1.0

## Known Issues

- PyO3 0.20 officially supports Python 3.7-3.12, use `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` for Python 3.13
- First model load is slow (~1-5 seconds), subsequent calls are fast
- Summarization model (BART) is large (~1.6GB), consider alternatives for low-memory systems

## Next Steps

See [TODO.md](./TODO.md) for:
- Alternative model support (ONNX, OpenAI API)
- Model caching strategies
- GPU support
- Batch inference optimization

## References

- Weaver integration: [../weaver/README.md](../weaver/README.md)
- Python ML guide: [../PYTHON_INTEGRATION.md](../PYTHON_INTEGRATION.md)
- Architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)

