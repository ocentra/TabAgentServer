//! Model data value types (tensors, embeddings, etc.).

use crate::{
    Value, ValueType, ValueData, ValueInner,
    markers::ModelDataValueMarker,
    types::TensorDataType,
    DowncastableTarget,
};

/// Model data value type alias.
pub type ModelValue = Value<ModelDataValueMarker>;

/// Concrete model data types (RAG: Use enums for type safety).
#[derive(Debug, Clone)]
pub enum ModelDataType {
    /// Tensor data (for ONNX, GGUF inference).
    Tensor {
        data: TensorData,
        dtype: TensorDataType,
        shape: Vec<i64>,
    },

    /// Embedding vector.
    Embedding {
        data: Vec<f32>,
    },

    /// Model parameters/weights.
    Parameters {
        data: Vec<u8>,
    },

    /// Tokenizer data.
    Tokenizer {
        vocab: Vec<String>,
        special_tokens: Vec<String>,
    },
}

/// Tensor data storage (RAG: Use enums to handle different types safely).
#[derive(Debug, Clone)]
pub enum TensorData {
    F32(Vec<f32>),
    F64(Vec<f64>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    U8(Vec<u8>),
    Bool(Vec<bool>),
}

impl ModelValue {
    /// Create a tensor value.
    pub fn tensor(data: TensorData, dtype: TensorDataType, shape: Vec<i64>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::ModelData(Box::new(ModelDataType::Tensor {
                    data,
                    dtype,
                    shape: shape.clone(),
                })),
                dtype: ValueType::Tensor { dtype, shape },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create an embedding value.
    pub fn embedding(data: Vec<f32>) -> Self {
        let dimensions = data.len();
        Value {
            inner: ValueInner {
                data: ValueData::ModelData(Box::new(ModelDataType::Embedding { data })),
                dtype: ValueType::Embedding { dimensions },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the underlying model data.
    pub fn model_data(&self) -> &ModelDataType {
        match &self.inner.data {
            ValueData::ModelData(data) => data,
            _ => unreachable!("ModelValue always contains ModelData"),
        }
    }
}

impl DowncastableTarget for ModelDataValueMarker {
    fn can_downcast(dtype: &ValueType) -> bool {
        matches!(
            dtype,
            ValueType::Tensor { .. }
                | ValueType::Embedding { .. }
                | ValueType::ModelParameters
                | ValueType::TokenizerData
        )
    }

    fn type_name() -> &'static str {
        "ModelDataValue"
    }
}


