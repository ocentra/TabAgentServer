//! Integration tests for tokenization
//!
//! These tests create a minimal tokenizer for testing purposes
//! without requiring external model downloads.

use tabagent_tokenization::{Tokenizer, TokenizationError};
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

/// Create a minimal test tokenizer.json for basic testing
fn create_test_tokenizer() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let tokenizer_path = temp_dir.path().join("tokenizer.json");
    
    // Minimal BPE tokenizer with basic vocabulary
    let tokenizer_json = r###"{
  "version": "1.0",
  "truncation": null,
  "padding": null,
  "added_tokens": [
    {
      "id": 0,
      "content": "[PAD]",
      "single_word": false,
      "lstrip": false,
      "rstrip": false,
      "normalized": false,
      "special": true
    },
    {
      "id": 1,
      "content": "[UNK]",
      "single_word": false,
      "lstrip": false,
      "rstrip": false,
      "normalized": false,
      "special": true
    },
    {
      "id": 2,
      "content": "[CLS]",
      "single_word": false,
      "lstrip": false,
      "rstrip": false,
      "normalized": false,
      "special": true
    },
    {
      "id": 3,
      "content": "[SEP]",
      "single_word": false,
      "lstrip": false,
      "rstrip": false,
      "normalized": false,
      "special": true
    }
  ],
  "normalizer": {
    "type": "Sequence",
    "normalizers": [
      {
        "type": "NFD"
      },
      {
        "type": "Lowercase"
      },
      {
        "type": "StripAccents"
      }
    ]
  },
  "pre_tokenizer": {
    "type": "Whitespace"
  },
  "post_processor": {
    "type": "TemplateProcessing",
    "single": [
      {
        "SpecialToken": {
          "id": "[CLS]",
          "type_id": 0
        }
      },
      {
        "Sequence": {
          "id": "A",
          "type_id": 0
        }
      },
      {
        "SpecialToken": {
          "id": "[SEP]",
          "type_id": 0
        }
      }
    ],
    "pair": [
      {
        "SpecialToken": {
          "id": "[CLS]",
          "type_id": 0
        }
      },
      {
        "Sequence": {
          "id": "A",
          "type_id": 0
        }
      },
      {
        "SpecialToken": {
          "id": "[SEP]",
          "type_id": 0
        }
      },
      {
        "Sequence": {
          "id": "B",
          "type_id": 1
        }
      },
      {
        "SpecialToken": {
          "id": "[SEP]",
          "type_id": 1
        }
      }
    ],
    "special_tokens": {
      "[CLS]": {
        "id": "[CLS]",
        "ids": [2],
        "tokens": ["[CLS]"]
      },
      "[SEP]": {
        "id": "[SEP]",
        "ids": [3],
        "tokens": ["[SEP]"]
      }
    }
  },
  "decoder": {
    "type": "WordPiece",
    "prefix": "##",
    "cleanup": true
  },
  "model": {
    "type": "WordPiece",
    "unk_token": "[UNK]",
    "continuing_subword_prefix": "##",
    "max_input_chars_per_word": 100,
    "vocab": {
      "[PAD]": 0,
      "[UNK]": 1,
      "[CLS]": 2,
      "[SEP]": 3,
      "hello": 4,
      "world": 5,
      "test": 6,
      "token": 7,
      "##ization": 8,
      "rust": 9,
      "fast": 10,
      "##er": 11
    }
  }
}"###;
    
    fs::write(&tokenizer_path, tokenizer_json)
        .expect("Failed to write tokenizer.json");
    
    (temp_dir, tokenizer_path)
}

#[test]
fn test_tokenizer_from_file() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    
    let tokenizer = Tokenizer::from_file(&tokenizer_path);
    assert!(tokenizer.is_ok(), "Should load tokenizer from file");
}

#[test]
fn test_tokenizer_from_nonexistent_file() {
    let result = Tokenizer::from_file("nonexistent_tokenizer.json");
    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[test]
fn test_tokenizer_from_pretrained_unsupported() {
    let result = Tokenizer::from_pretrained("bert-base-uncased", None);
    assert!(result.is_err(), "from_pretrained should not be supported");
    
    if let Err(TokenizationError::LoadFailed(msg)) = result {
        assert!(msg.contains("not supported"), "Error message should mention not supported");
    } else {
        panic!("Expected LoadFailed error");
    }
}

#[test]
fn test_encode_basic() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let encoding = tokenizer.encode("hello world", true)
        .expect("Failed to encode text");
    
    let ids = encoding.get_ids();
    assert!(!ids.is_empty(), "Should produce token IDs");
    assert!(ids.contains(&2), "Should contain [CLS] token");
    assert!(ids.contains(&3), "Should contain [SEP] token");
}

#[test]
fn test_encode_without_special_tokens() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let encoding = tokenizer.encode("hello world", false)
        .expect("Failed to encode text");
    
    let ids = encoding.get_ids();
    assert!(!ids.is_empty(), "Should produce token IDs");
}

#[test]
fn test_encode_batch() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let texts = vec!["hello world", "test tokenization"];
    let encodings = tokenizer.encode_batch(texts, true)
        .expect("Failed to encode batch");
    
    assert_eq!(encodings.len(), 2, "Should return 2 encodings");
    for encoding in &encodings {
        let ids = encoding.get_ids();
        assert!(!ids.is_empty(), "Each encoding should have token IDs");
    }
}

#[test]
fn test_decode_basic() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    // Encode first
    let encoding = tokenizer.encode("hello world", true)
        .expect("Failed to encode");
    let ids = encoding.get_ids();
    
    // Then decode
    let decoded = tokenizer.decode(ids, true)
        .expect("Failed to decode");
    
    assert!(!decoded.is_empty(), "Decoded text should not be empty");
}

#[test]
fn test_decode_with_special_tokens() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let ids = vec![2, 4, 5, 3]; // [CLS], hello, world, [SEP]
    
    let decoded_with_special = tokenizer.decode(&ids, false)
        .expect("Failed to decode with special tokens");
    
    let decoded_without_special = tokenizer.decode(&ids, true)
        .expect("Failed to decode without special tokens");
    
    // Without special tokens should be shorter or different
    assert!(!decoded_with_special.is_empty());
    assert!(!decoded_without_special.is_empty());
}

#[test]
fn test_decode_batch() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let seq1 = vec![2, 4, 5, 3];  // [CLS], hello, world, [SEP]
    let seq2 = vec![2, 6, 7, 3];  // [CLS], test, token, [SEP]
    let sequences = vec![
        seq1.as_slice(),
        seq2.as_slice(),
    ];
    
    let decoded = tokenizer.decode_batch(&sequences, true)
        .expect("Failed to decode batch");
    
    assert_eq!(decoded.len(), 2, "Should decode 2 sequences");
    for text in &decoded {
        assert!(!text.is_empty(), "Each decoded text should not be empty");
    }
}

#[test]
fn test_vocab_size() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let vocab_size = tokenizer.vocab_size();
    assert!(vocab_size > 0, "Vocab size should be positive");
    assert_eq!(vocab_size, 12, "Test tokenizer should have 12 tokens");
}

#[test]
fn test_special_token_ids() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    // Test BOS token (should be CLS in this tokenizer)
    let bos = tokenizer.bos_token_id();
    assert!(bos.is_some(), "Should have BOS token");
    assert_eq!(bos.unwrap(), 2, "BOS should be [CLS] with ID 2");
    
    // Test EOS token (should be SEP in this tokenizer)
    let eos = tokenizer.eos_token_id();
    assert!(eos.is_some(), "Should have EOS token");
    assert_eq!(eos.unwrap(), 3, "EOS should be [SEP] with ID 3");
    
    // Test PAD token
    let pad = tokenizer.pad_token_id();
    assert!(pad.is_some(), "Should have PAD token");
    assert_eq!(pad.unwrap(), 0, "PAD should have ID 0");
    
    // Test UNK token
    let unk = tokenizer.unk_token_id();
    assert!(unk.is_some(), "Should have UNK token");
    assert_eq!(unk.unwrap(), 1, "UNK should have ID 1");
}

#[test]
fn test_round_trip_encoding() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let original_text = "hello world";
    
    // Encode
    let encoding = tokenizer.encode(original_text, false)
        .expect("Failed to encode");
    let ids = encoding.get_ids();
    
    // Decode
    let decoded_text = tokenizer.decode(ids, true)
        .expect("Failed to decode");
    
    // Should be similar (may have whitespace differences)
    assert!(decoded_text.contains("hello"), "Should contain 'hello'");
    assert!(decoded_text.contains("world"), "Should contain 'world'");
}

#[test]
fn test_empty_text_encoding() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let encoding = tokenizer.encode("", true)
        .expect("Should handle empty text");
    
    let ids = encoding.get_ids();
    // With special tokens, should still have [CLS] and [SEP]
    assert!(!ids.is_empty(), "Should have special tokens even for empty text");
}

#[test]
fn test_unknown_tokens() {
    let (_temp_dir, tokenizer_path) = create_test_tokenizer();
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    // Text with unknown tokens (not in our small vocab)
    let encoding = tokenizer.encode("unknown words xyz", true)
        .expect("Should handle unknown tokens");
    
    let ids = encoding.get_ids();
    assert!(!ids.is_empty(), "Should produce token IDs");
    // Should contain UNK token (ID 1) for unknown words
}

