# TabAgent Tokenization

Pure Rust tokenization using HuggingFace's `tokenizers` library.

## Features

- **Fast Tokenization**: Rust-native performance via HuggingFace tokenizers
- **Unicode Support**: Full Unicode normalization and processing
- **Batch Processing**: Efficient batch encoding/decoding
- **Error Handling**: Comprehensive error types with `thiserror`

## Architecture

```
tokenization/
├── src/
│   ├── lib.rs       # Public API, Tokenizer wrapper
│   └── error.rs     # Error types
└── tests/
    └── tokenizer_test.rs  # Integration tests
```

## Usage

```rust
use tabagent_tokenization::Tokenizer;

// Load from file
let tokenizer = Tokenizer::from_file("tokenizer.json")?;

// Encode text
let encoding = tokenizer.encode("Hello world!", true)?;
let token_ids = encoding.get_ids();

// Decode tokens
let text = tokenizer.decode(token_ids, true)?;

// Batch decode
let texts = tokenizer.decode_batch(&[ids1, ids2], true)?;

// Vocab size
let size = tokenizer.vocab_size();
```

## Dependencies

- **tokenizers**: HuggingFace's fast tokenizers library (v0.19)
- **thiserror**: Ergonomic error handling
- **serde**: Serialization support

## Integration

Used by:
- `onnx-loader`: Text model tokenization
- `pipeline`: High-level tokenization API

## Testing

Integration tests with real tokenizer files from HuggingFace models.

```bash
cargo test -p tabagent-tokenization
```

