# Pipelines - HuggingFace Transformers

**15 specialized ML pipelines for text, audio, and vision**

Factory-based system for creating and managing HuggingFace Transformers pipelines. Uses `RustFileProvider` to fetch model files from Rust's ModelCache.

---

## Architecture

```
PipelineFactory.create_pipeline(task, model_id, architecture)
    ↓
Selects appropriate pipeline class
    ↓
BasePipeline subclass (e.g., Florence2Pipeline)
    ↓
Sets file_provider (RustFileProvider)
    ↓
Loads model using HuggingFace Transformers
    ↓
Ready for inference
```

**Key Principle**: Pipelines request model files → RustFileProvider fetches from Rust → No duplicate downloads

---

## Available Pipelines

### Text Generation (`text_generation.py`)
**Task**: `text-generation`  
**Models**: GPT-2, LLaMA, Mistral, Phi, Qwen, etc.  
**Use Cases**: Text completion, creative writing, code generation

```python
from pipelines import PipelineFactory

pipeline = PipelineFactory.create_pipeline(
    task="text-generation",
    model_id="meta-llama/Llama-2-7b-hf"
)
pipeline.file_provider = rust_file_provider
pipeline.load(model_id="meta-llama/Llama-2-7b-hf")

result = pipeline.generate({
    "prompt": "Once upon a time",
    "max_new_tokens": 50,
    "temperature": 0.7
})
print(result['text'])
```

---

### Embeddings (`embedding.py`)
**Task**: `feature-extraction`  
**Models**: sentence-transformers, BERT, RoBERTa  
**Use Cases**: Semantic search, similarity, clustering

```python
pipeline = PipelineFactory.create_pipeline(
    task="feature-extraction",
    model_id="sentence-transformers/all-MiniLM-L6-v2"
)
pipeline.load(model_id="sentence-transformers/all-MiniLM-L6-v2")

result = pipeline.generate({
    "texts": ["Hello world", "Machine learning"],
    "normalize_embeddings": True
})
embeddings = result['embeddings']  # List[List[float]]
```

---

### Speech-to-Text (`whisper.py`)
**Task**: `automatic-speech-recognition`  
**Models**: Whisper (tiny, base, small, medium, large)  
**Use Cases**: Transcription, captioning, voice commands

```python
pipeline = PipelineFactory.create_pipeline(
    task="whisper",
    model_id="openai/whisper-base"
)
pipeline.load(model_id="openai/whisper-base")

result = pipeline.generate({
    "audio": audio_array,  # numpy array
    "language": "en",  # optional
    "task": "transcribe"  # or "translate"
})
print(result['text'])
```

---

### Vision-Language (`florence2.py`)
**Task**: `florence2`  
**Models**: Florence-2 (base, large)  
**Use Cases**: Image captioning, VQA, object detection, OCR

```python
pipeline = PipelineFactory.create_pipeline(
    task="florence2",
    model_id="microsoft/Florence-2-base"
)
pipeline.load(model_id="microsoft/Florence-2-base")

result = pipeline.generate({
    "image": pil_image,
    "prompt": "<OD>",  # Object detection
    "max_new_tokens": 1024
})
print(result['text'])
```

---

### Image-Text (`clip.py`)
**Task**: `clip`  
**Models**: CLIP (ViT, ResNet variants)  
**Use Cases**: Zero-shot classification, image search, multimodal embeddings

```python
pipeline = PipelineFactory.create_pipeline(
    task="clip",
    model_id="openai/clip-vit-base-patch32"
)
pipeline.load(model_id="openai/clip-vit-base-patch32")

result = pipeline.generate({
    "image": pil_image,
    "texts": ["a cat", "a dog", "a bird"]
})
probs = result['probabilities']  # [0.7, 0.2, 0.1]
```

---

### Audio-Text (`clap.py`)
**Task**: `clap`  
**Models**: CLAP  
**Use Cases**: Audio classification, sound search

---

### Translation (`translation.py`)
**Task**: `translation`  
**Models**: MarianMT, NLLB, M2M100  
**Use Cases**: Language translation

---

### Code Completion (`code_completion.py`)
**Task**: `code-generation`  
**Models**: CodeLLaMA, StarCoder, CodeGen  
**Use Cases**: Code completion, generation, refactoring

---

### Cross-Encoder (`cross_encoder.py`)
**Task**: `text-similarity`  
**Models**: Cross-encoder models  
**Use Cases**: Re-ranking, semantic similarity

---

### Image Classification (`image_classification.py`)
**Task**: `image-classification`  
**Models**: ViT, ResNet, EfficientNet  
**Use Cases**: Image classification, object recognition

---

### Multimodal (`multimodal.py`)
**Task**: `multimodal`  
**Models**: LLaVA, Qwen-VL, etc.  
**Use Cases**: Visual question answering, image reasoning

---

### Janus (`janus.py`)
**Task**: `janus`  
**Models**: Janus (multimodal understanding)  
**Use Cases**: Unified vision-language tasks

---

### Text-to-Speech (`text_to_speech.py`)
**Task**: `text-to-speech`  
**Models**: Bark, VITS, FastSpeech  
**Use Cases**: Speech synthesis

---

### Tokenizer (`tokenizer.py`)
**Task**: `tokenization`  
**Purpose**: Tokenization utilities  
**Use Cases**: Token counting, encoding/decoding

---

### Zero-Shot Classification (`zero_shot_classification.py`)
**Task**: `zero-shot-classification`  
**Models**: BART, DeBERTa  
**Use Cases**: Classify without training

---

## Factory Pattern

**`factory.py`** - Smart pipeline creation

```python
class PipelineFactory:
    @staticmethod
    def create_pipeline(
        task: str,
        model_id: str,
        architecture: Optional[str] = None
    ) -> BasePipeline:
        """
        Create pipeline based on task or architecture.
        
        Priority:
        1. Architecture (if provided)
        2. Task type
        3. Model ID patterns
        """
```

**Examples**:
```python
# By task
pipeline = PipelineFactory.create_pipeline(
    task="text-generation",
    model_id="gpt2"
)

# By architecture
pipeline = PipelineFactory.create_pipeline(
    task="multimodal",
    model_id="microsoft/Florence-2-base",
    architecture="Florence2"
)

# Auto-detect from model ID
pipeline = PipelineFactory.create_pipeline(
    task="feature-extraction",
    model_id="sentence-transformers/all-MiniLM-L6-v2"
)
```

---

## Base Pipeline

**`base.py`** - Abstract base class

```python
class BasePipeline(ABC):
    def __init__(self):
        self.model = None
        self.tokenizer = None
        self.processor = None
        self.file_provider: Optional[RustFileProvider] = None
    
    @abstractmethod
    def pipeline_type(self) -> str:
        """Return pipeline task type"""
    
    @abstractmethod
    def load(self, model_id: str, options: dict) -> dict:
        """Load model, returns status"""
    
    @abstractmethod
    def generate(self, input_data: dict) -> dict:
        """Run inference, returns results"""
    
    def unload(self):
        """Free resources"""
```

---

## File Provider Integration

**How It Works**:

1. Pipeline calls `transformers.AutoModel.from_pretrained(model_id)`
2. Transformers tries to download `config.json`
3. RustFileProvider intercepts (via custom resolver)
4. Fetches file from Rust via gRPC
5. Transformers continues loading with local file

**Setting File Provider**:
```python
pipeline = PipelineFactory.create_pipeline(...)
pipeline.file_provider = rust_file_provider  # Set before load()
pipeline.load(model_id="...")
```

---

## Pipeline Types

**Defined in `types.py`**:

```python
class PipelineTask(str, Enum):
    TEXT_GENERATION = "text-generation"
    FEATURE_EXTRACTION = "feature-extraction"
    AUTOMATIC_SPEECH_RECOGNITION = "automatic-speech-recognition"
    IMAGE_TO_TEXT = "image-to-text"
    IMAGE_CLASSIFICATION = "image-classification"
    OBJECT_DETECTION = "object-detection"
    ZERO_SHOT_CLASSIFICATION = "zero-shot-classification"
    TRANSLATION = "translation"
    # ... all task types
```

---

## Testing

```bash
# Unit tests
pytest tests/test_pipelines.py -v

# Integration tests (requires Rust + models)
pytest tests/test_pipelines_integration.py -v

# Test specific pipeline
pytest tests/test_pipelines.py::TestTextGeneration -v
```

---

## Adding a New Pipeline

1. **Create pipeline file**: `pipelines/my_pipeline.py`

```python
from .base import BasePipeline

class MyPipeline(BasePipeline):
    def pipeline_type(self) -> str:
        return "my-task"
    
    def load(self, model_id: str, options: dict) -> dict:
        # Use self.file_provider to fetch files
        # Load model with transformers
        # Return {"status": "success"}
        pass
    
    def generate(self, input_data: dict) -> dict:
        # Run inference
        # Return results
        pass
```

2. **Add to `__init__.py`**:
```python
from .my_pipeline import MyPipeline
```

3. **Register in `factory.py`**:
```python
TASK_TO_PIPELINE = {
    PipelineTask.MY_TASK: MyPipeline,
    # ...
}
```

4. **Add to `types.py`**:
```python
class PipelineTask(str, Enum):
    MY_TASK = "my-task"
```

---

## Status

⚙️ **In Progress**: Full implementation of all 15 pipelines  
✅ **Structure**: Complete factory + base + types  
⚙️ **Integration**: RustFileProvider wiring

**See**: [TODO.md](TODO.md) for detailed status

---

## See Also

- **[Core](../core/README.md)** - RustFileProvider implementation
- **[Services](../services/README.md)** - TransformersService usage
- **[ModelCache (Rust)](../../Rust/model-cache/README.md)** - File serving

