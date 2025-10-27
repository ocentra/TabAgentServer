//! Comprehensive tests for model data values (tensors, embeddings, etc.).

use tabagent_values::{
    ModelValue, TensorData, TensorDataType, ValueType,
};

#[test]
fn test_embedding_creation() {
    let embedding = ModelValue::embedding(vec![0.1, 0.2, 0.3]);
    
    assert!(matches!(
        embedding.value_type(),
        ValueType::Embedding { dimensions: 3 }
    ));
}

#[test]
fn test_embedding_large_dimensions() {
    let data: Vec<f32> = (0..1536).map(|i| i as f32 * 0.001).collect();
    let embedding = ModelValue::embedding(data);
    
    assert!(matches!(
        embedding.value_type(),
        ValueType::Embedding { dimensions: 1536 }
    ));
}

#[test]
fn test_embedding_zero_dimensions() {
    let embedding = ModelValue::embedding(vec![]);
    
    assert!(matches!(
        embedding.value_type(),
        ValueType::Embedding { dimensions: 0 }
    ));
}

#[test]
fn test_tensor_f32_creation() {
    let tensor = ModelValue::tensor(
        TensorData::F32(vec![1.0, 2.0, 3.0, 4.0]),
        TensorDataType::Float32,
        vec![2, 2],
    );

    assert!(matches!(
        tensor.value_type(),
        ValueType::Tensor { dtype: TensorDataType::Float32, .. }
    ));
}

#[test]
fn test_tensor_i32_creation() {
    let tensor = ModelValue::tensor(
        TensorData::I32(vec![1, 2, 3, 4]),
        TensorDataType::Int32,
        vec![2, 2],
    );

    assert!(matches!(
        tensor.value_type(),
        ValueType::Tensor { dtype: TensorDataType::Int32, .. }
    ));
}

#[test]
fn test_tensor_bool_creation() {
    let tensor = ModelValue::tensor(
        TensorData::Bool(vec![true, false, true, false]),
        TensorDataType::Bool,
        vec![2, 2],
    );

    assert!(matches!(
        tensor.value_type(),
        ValueType::Tensor { dtype: TensorDataType::Bool, .. }
    ));
}

#[test]
fn test_tensor_1d_shape() {
    let tensor = ModelValue::tensor(
        TensorData::F32(vec![1.0, 2.0, 3.0]),
        TensorDataType::Float32,
        vec![3],
    );

    if let ValueType::Tensor { shape, .. } = tensor.value_type() {
        assert_eq!(shape, &vec![3]);
    } else {
        panic!("Expected Tensor value type");
    }
}

#[test]
fn test_tensor_3d_shape() {
    let data: Vec<f32> = (0..24).map(|i| i as f32).collect();
    let tensor = ModelValue::tensor(
        TensorData::F32(data),
        TensorDataType::Float32,
        vec![2, 3, 4],
    );

    if let ValueType::Tensor { shape, .. } = tensor.value_type() {
        assert_eq!(shape, &vec![2, 3, 4]);
    } else {
        panic!("Expected Tensor value type");
    }
}

#[test]
fn test_tensor_4d_shape_batch() {
    let data: Vec<f32> = (0..192).map(|i| i as f32).collect();
    let tensor = ModelValue::tensor(
        TensorData::F32(data),
        TensorDataType::Float32,
        vec![2, 3, 4, 8],  // batch, channels, height, width
    );

    if let ValueType::Tensor { shape, .. } = tensor.value_type() {
        assert_eq!(shape, &vec![2, 3, 4, 8]);
    } else {
        panic!("Expected Tensor value type");
    }
}

#[test]
fn test_tensor_data_access() {
    use tabagent_values::ModelDataType;
    
    let tensor = ModelValue::tensor(
        TensorData::F32(vec![1.0, 2.0, 3.0]),
        TensorDataType::Float32,
        vec![3],
    );

    match tensor.model_data() {
        ModelDataType::Tensor { data, dtype, shape } => {
            assert!(matches!(data, TensorData::F32(_)));
            assert_eq!(*dtype, TensorDataType::Float32);
            assert_eq!(shape, &vec![3]);
        }
        _ => panic!("Expected Tensor model data"),
    }
}

#[test]
fn test_embedding_data_access() {
    use tabagent_values::ModelDataType;
    
    let embedding = ModelValue::embedding(vec![0.1, 0.2, 0.3]);

    match embedding.model_data() {
        ModelDataType::Embedding { data } => {
            assert_eq!(data.len(), 3);
            assert_eq!(data[0], 0.1);
            assert_eq!(data[1], 0.2);
            assert_eq!(data[2], 0.3);
        }
        _ => panic!("Expected Embedding model data"),
    }
}

#[test]
fn test_tensor_dtype_variants() {
    let dtypes = vec![
        TensorDataType::Float16,
        TensorDataType::Float32,
        TensorDataType::Float64,
        TensorDataType::Int8,
        TensorDataType::Int16,
        TensorDataType::Int32,
        TensorDataType::Int64,
        TensorDataType::Uint8,
        TensorDataType::Uint16,
        TensorDataType::Uint32,
        TensorDataType::Uint64,
        TensorDataType::Bool,
        TensorDataType::String,
    ];

    for dtype in dtypes {
        // Just verify they all exist and can be used
        let _ = serde_json::to_string(&dtype).unwrap();
    }
}

#[test]
fn test_embedding_normalization_values() {
    // Test common embedding value ranges
    let embedding = ModelValue::embedding(vec![-1.0, -0.5, 0.0, 0.5, 1.0]);
    
    assert!(matches!(
        embedding.value_type(),
        ValueType::Embedding { dimensions: 5 }
    ));
}

#[test]
fn test_large_tensor() {
    // Test with a large tensor (common in ML)
    let size = 512 * 512;
    let data: Vec<f32> = (0..size).map(|i| i as f32).collect();
    let tensor = ModelValue::tensor(
        TensorData::F32(data),
        TensorDataType::Float32,
        vec![512, 512],
    );

    if let ValueType::Tensor { shape, .. } = tensor.value_type() {
        assert_eq!(shape, &vec![512, 512]);
    }
}

