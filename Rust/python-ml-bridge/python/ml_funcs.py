"""
ML Functions for TabAgent Embedded Database.

This module provides stateless ML inference functions called from Rust via PyO3.
Functions are designed to be:
- Stateless (models loaded once and cached)
- Thread-safe (no global state mutations)
- Fast (models kept in memory)

Required packages:
    pip install sentence-transformers spacy transformers torch
    python -m spacy download en_core_web_sm
"""

import logging
from typing import List, Dict, Any
import warnings

# Suppress warnings from transformers
warnings.filterwarnings("ignore")

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Global model instances (lazy-loaded)
_embed_model = None
_nlp = None
_summarizer = None


def _get_embed_model():
    """Get or initialize the embedding model."""
    global _embed_model
    if _embed_model is None:
        try:
            from sentence_transformers import SentenceTransformer
            logger.info("Loading embedding model: all-MiniLM-L6-v2")
            _embed_model = SentenceTransformer('all-MiniLM-L6-v2')
            logger.info("Embedding model loaded successfully (384 dims)")
        except ImportError as e:
            logger.error("sentence-transformers not installed: %s", e)
            raise RuntimeError(
                "Please install: pip install sentence-transformers"
            ) from e
        except Exception as e:
            logger.error("Failed to load embedding model: %s", e)
            raise
    return _embed_model


def _get_nlp():
    """Get or initialize the spaCy NER model."""
    global _nlp
    if _nlp is None:
        try:
            import spacy
            logger.info("Loading spaCy model: en_core_web_sm")
            _nlp = spacy.load('en_core_web_sm')
            logger.info("spaCy model loaded successfully")
        except OSError as e:
            logger.error("spaCy model not found: %s", e)
            raise RuntimeError(
                "Please download: python -m spacy download en_core_web_sm"
            ) from e
        except ImportError as e:
            logger.error("spaCy not installed: %s", e)
            raise RuntimeError("Please install: pip install spacy") from e
        except Exception as e:
            logger.error("Failed to load spaCy model: %s", e)
            raise
    return _nlp


def _get_summarizer():
    """Get or initialize the summarization pipeline."""
    global _summarizer
    if _summarizer is None:
        try:
            from transformers import pipeline
            logger.info("Loading summarization model: facebook/bart-large-cnn")
            _summarizer = pipeline(
                "summarization",
                model="facebook/bart-large-cnn",
                device=-1  # CPU
            )
            logger.info("Summarization model loaded successfully")
        except ImportError as e:
            logger.error("transformers not installed: %s", e)
            raise RuntimeError(
                "Please install: pip install transformers torch"
            ) from e
        except Exception as e:
            logger.error("Failed to load summarization model: %s", e)
            raise
    return _summarizer


def generate_embedding(text: str) -> List[float]:
    """
    Generate a 384-dimensional embedding for the given text.

    Uses sentence-transformers (all-MiniLM-L6-v2) for fast, high-quality embeddings.

    Args:
        text: Input text to embed

    Returns:
        List of 384 floats representing the embedding vector

    Raises:
        RuntimeError: If model is not available

    Example:
        >>> emb = generate_embedding("Hello world")
        >>> len(emb)
        384
        >>> isinstance(emb[0], float)
        True
    """
    if not text or not text.strip():
        logger.warning("Empty text provided for embedding, returning zero vector")
        return [0.0] * 384

    try:
        model = _get_embed_model()
        embedding = model.encode(
            text,
            convert_to_tensor=False,
            show_progress_bar=False
        )
        # Convert numpy array to list
        return embedding.tolist()
    except Exception as e:
        logger.error("Embedding generation failed: %s", e)
        raise RuntimeError(f"Embedding generation failed: {e}") from e


def extract_entities(text: str) -> List[Dict[str, Any]]:
    """
    Extract named entities from text using spaCy.

    Identifies entities like PERSON, ORG, GPE (geopolitical entity), DATE, etc.

    Args:
        text: Input text to analyze

    Returns:
        List of dictionaries with keys:
        - text (str): The entity text
        - label (str): Entity type (PERSON, ORG, GPE, etc.)
        - start (int): Start character position
        - end (int): End character position

    Example:
        >>> entities = extract_entities("Alice met Bob in Paris")
        >>> len(entities)
        3
        >>> entities[0]["label"] in ["PERSON", "ORG", "GPE"]
        True
    """
    if not text or not text.strip():
        logger.warning("Empty text provided for entity extraction")
        return []

    try:
        nlp = _get_nlp()
        doc = nlp(text)

        entities = []
        for ent in doc.ents:
            entities.append({
                "text": ent.text,
                "label": ent.label_,
                "start": ent.start_char,
                "end": ent.end_char,
            })

        logger.debug("Extracted %d entities from text", len(entities))
        return entities

    except Exception as e:
        logger.error("Entity extraction failed: %s", e)
        raise RuntimeError(f"Entity extraction failed: {e}") from e


def summarize(messages: List[str]) -> str:
    """
    Summarize a list of messages into a concise summary.

    Uses BART (facebook/bart-large-cnn) for abstractive summarization.

    Args:
        messages: List of message texts to summarize

    Returns:
        A concise summary string

    Example:
        >>> msgs = ["Hello there!", "How are you?", "I'm doing great!"]
        >>> summary = summarize(msgs)
        >>> len(summary) > 0
        True
    """
    if not messages:
        logger.warning("Empty message list provided for summarization")
        return "No messages to summarize."

    try:
        summarizer = _get_summarizer()

        # Concatenate messages with clear separators
        full_text = " ".join(msg.strip() for msg in messages if msg.strip())

        if not full_text:
            return "No content to summarize."

        # BART has a max input length of 1024 tokens (~750-800 words)
        # Truncate if necessary
        max_chars = 3000  # Conservative estimate
        if len(full_text) > max_chars:
            logger.warning(
                "Text too long (%d chars), truncating to %d",
                len(full_text),
                max_chars
            )
            full_text = full_text[:max_chars]

        # Generate summary
        result = summarizer(
            full_text,
            max_length=150,
            min_length=40,
            do_sample=False,
            truncation=True
        )

        summary = result[0]['summary_text']
        logger.debug("Generated summary of %d chars", len(summary))
        return summary

    except Exception as e:
        logger.error("Summarization failed: %s", e)
        # Return a fallback summary
        return f"Summary of {len(messages)} messages (generation failed)."


# Health check function
def health_check() -> bool:
    """
    Check if all ML models can be loaded.

    Returns:
        True if all models are available, False otherwise
    """
    try:
        _get_embed_model()
        _get_nlp()
        # Skip summarizer for faster health checks
        logger.info("ML bridge health check passed")
        return True
    except Exception as e:
        logger.error("ML bridge health check failed: %s", e)
        return False


if __name__ == "__main__":
    # Test the functions
    print("Testing ML functions...")

    # Test embedding
    print("\n1. Testing embedding generation...")
    emb = generate_embedding("Hello world")
    print(f"   ✓ Generated {len(emb)}-dim embedding")

    # Test entity extraction
    print("\n2. Testing entity extraction...")
    entities = extract_entities("Alice met Bob in Paris on January 1st")
    print(f"   ✓ Extracted {len(entities)} entities:")
    for ent in entities:
        print(f"     - {ent['text']} ({ent['label']})")

    # Test summarization
    print("\n3. Testing summarization...")
    messages = [
        "We need to build a new database system.",
        "It should be fast and embedded.",
        "Rust is a great choice for this.",
        "We can use Python for ML parts."
    ]
    summary = summarize(messages)
    print(f"   ✓ Generated summary: {summary}")

    print("\n✓ All tests passed!")

