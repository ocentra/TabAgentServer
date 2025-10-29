//! # TabAgent Values
//!
//! Unified value system for TabAgent providing type-safe request/response/data handling.
//!
//! Inspired by ONNX Runtime's value system, this crate provides:
//! - **Type markers** for compile-time safety
//! - **Owned and borrowed variants** for zero-copy operations
//! - **Downcasting support** from dynamic to concrete types
//! - **Extensible type system** for all TabAgent data types
//!
//! ## Architecture
//!
//! ```text
//! Value<Type>         <- Owned value (like Box)
//!    ↓
//! ValueRef<'v, Type>  <- Borrowed value (like &)
//!    ↓
//! ValueRefMut<'v, Type> <- Mutable borrowed (like &mut)
//! ```
//!
//! ## Usage Example
//!
//! ```rust
//! use tabagent_values::{Value, ValueType, RequestValue, ResponseValue};
//!
//! // Create a request value
//! let request = RequestValue::chat(
//!     "gpt-3.5-turbo",
//!     vec![("user", "Hello!")],
//!     None,
//! );
//!
//! // Type-safe access
//! if let ValueType::ChatRequest { model, messages, .. } = request.value_type() {
//!     println!("Model: {}", model);
//! }
//! ```

pub mod error;
pub mod types;
pub mod request;
pub mod response;
pub mod model_data;
pub mod markers;

// Re-exports for convenience
pub use error::{ValueError, ValueResult, BackendError, BackendResult};
pub use types::{ValueType, TensorDataType};
pub use request::{RequestValue, RequestType, Message, MessageRole, EmbeddingInput};
pub use response::{ResponseValue, ResponseType, TokenUsage, FinishReason, HealthStatus, ModelInfo};
pub use model_data::{ModelValue, ModelDataType, TensorData};
pub use markers::{ValueTypeMarker, DynValueTypeMarker};

use std::fmt;

/// Core value wrapper providing type-safe access to TabAgent data.
///
/// Values can be:
/// - **Request values**: Chat, Generate, Embeddings, etc.
/// - **Response values**: ChatResponse, GenerateResponse, etc.
/// - **Model data**: Tensors, Embeddings, Parameters, etc.
///
/// # Type Safety
///
/// The `Type` parameter is a marker trait that ensures compile-time type safety:
///
/// ```rust
/// use tabagent_values::{Value, RequestValue, ResponseValue};
///
/// // Type is enforced at compile time
/// fn handle_request(req: Value<RequestValue>) {
///     // Can only be a request
/// }
///
/// fn handle_response(resp: Value<ResponseValue>) {
///     // Can only be a response
/// }
/// ```
#[derive(Clone)]
pub struct Value<Type: ValueTypeMarker = DynValueTypeMarker> {
    inner: ValueInner,
    _marker: std::marker::PhantomData<Type>,
}

#[derive(Clone)]
struct ValueInner {
    data: ValueData,
    dtype: ValueType,
}

/// The actual data stored in a value (RAG: Use enums for type safety)
#[derive(Clone)]
enum ValueData {
    Request(Box<RequestType>),
    Response(Box<ResponseType>),
    ModelData(Box<ModelDataType>),
}

impl<Type: ValueTypeMarker> Value<Type> {
    /// Get the type information for this value.
    pub fn value_type(&self) -> &ValueType {
        &self.inner.dtype
    }

    /// Attempt to downcast this value to a more specific type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tabagent_values::{Value, DynValueTypeMarker, RequestValue};
    ///
    /// let dyn_value: Value<DynValueTypeMarker> = todo!();
    /// let request: Value<RequestValue> = dyn_value.downcast()?;
    /// # Ok::<(), tabagent_values::ValueError>(())
    /// ```
    pub fn downcast<Target: ValueTypeMarker + DowncastableTarget>(self) -> ValueResult<Value<Target>> {
        if Target::can_downcast(&self.inner.dtype) {
            Ok(Value {
                inner: self.inner,
                _marker: std::marker::PhantomData,
            })
        } else {
            Err(ValueError::TypeMismatch {
                expected: Target::type_name(),
                actual: self.inner.dtype.clone(),
            })
        }
    }

    /// Convert to a dynamic (type-erased) value.
    pub fn into_dyn(self) -> Value<DynValueTypeMarker> {
        Value {
            inner: self.inner,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Type: ValueTypeMarker> fmt::Debug for Value<Type> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Value")
            .field("type", &self.inner.dtype)
            .field("marker", &std::any::type_name::<Type>())
            .finish()
    }
}

/// Borrowed reference to a value (zero-copy).
///
/// # Lifetime
///
/// The lifetime `'v` ensures the reference cannot outlive the original value.
///
/// # Example
///
/// ```rust
/// use tabagent_values::{Value, ValueRef, RequestValue};
///
/// fn process_request(req: ValueRef<'_, RequestValue>) {
///     // Borrowed access, no cloning
///     println!("Type: {:?}", req.value_type());
/// }
/// ```
pub struct ValueRef<'v, Type: ValueTypeMarker = DynValueTypeMarker> {
    inner: &'v ValueInner,
    _marker: std::marker::PhantomData<Type>,
}

impl<'v, Type: ValueTypeMarker> ValueRef<'v, Type> {
    /// Create a new borrowed reference from a value.
    pub fn new(value: &'v Value<Type>) -> Self {
        ValueRef {
            inner: &value.inner,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the type information.
    pub fn value_type(&self) -> &ValueType {
        &self.inner.dtype
    }

    /// Attempt to downcast to a more specific type.
    pub fn downcast<Target: ValueTypeMarker + DowncastableTarget>(self) -> ValueResult<ValueRef<'v, Target>> {
        if Target::can_downcast(&self.inner.dtype) {
            Ok(ValueRef {
                inner: self.inner,
                _marker: std::marker::PhantomData,
            })
        } else {
            Err(ValueError::TypeMismatch {
                expected: Target::type_name(),
                actual: self.inner.dtype.clone(),
            })
        }
    }
}

impl<Type: ValueTypeMarker> fmt::Debug for ValueRef<'_, Type> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValueRef")
            .field("type", &self.inner.dtype)
            .finish()
    }
}

/// Marker trait for downcasting support (RAG: Use traits for polymorphism).
pub trait DowncastableTarget: ValueTypeMarker {
    /// Check if a runtime type can be downcast to this marker type.
    fn can_downcast(dtype: &ValueType) -> bool;
    
    /// Get the name of this type for error messages.
    fn type_name() -> &'static str;
}


