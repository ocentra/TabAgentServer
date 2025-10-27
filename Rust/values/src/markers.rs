//! Type markers for compile-time type safety.
//!
//! These marker traits ensure that values can only be used in appropriate contexts.
//! Following the pattern from ort, we use zero-sized marker types.

/// Base trait for all value type markers.
///
/// # RAG: Type Safety
///
/// This trait ensures values are used correctly at compile time:
///
/// ```rust
/// use tabagent_values::{Value, RequestValue, ResponseValue};
///
/// // This function can ONLY accept requests
/// fn handle_request(req: Value<RequestValue>) {
///     // Type system guarantees req is a request
/// }
///
/// // This function can ONLY accept responses  
/// fn handle_response(resp: Value<ResponseValue>) {
///     // Type system guarantees resp is a response
/// }
/// ```
pub trait ValueTypeMarker: 'static + Send + Sync {}

/// Dynamic (type-erased) value marker.
///
/// Used when the concrete type is not known at compile time.
///
/// # Example
///
/// ```rust
/// use tabagent_values::{Value, DynValueTypeMarker};
///
/// fn handle_any_value(value: Value<DynValueTypeMarker>) {
///     // Can be any type, must check at runtime
///     match value.value_type() {
///         // Handle different types...
///         # _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DynValueTypeMarker;
impl ValueTypeMarker for DynValueTypeMarker {}

/// Marker for request values.
#[derive(Debug, Clone, Copy)]
pub struct RequestValueMarker;
impl ValueTypeMarker for RequestValueMarker {}

/// Marker for response values.
#[derive(Debug, Clone, Copy)]
pub struct ResponseValueMarker;
impl ValueTypeMarker for ResponseValueMarker {}

/// Marker for model data values (tensors, embeddings, etc.).
#[derive(Debug, Clone, Copy)]
pub struct ModelDataValueMarker;
impl ValueTypeMarker for ModelDataValueMarker {}

// Specific request type markers (for fine-grained type safety)

/// Marker for chat request values.
#[derive(Debug, Clone, Copy)]
pub struct ChatRequestMarker;
impl ValueTypeMarker for ChatRequestMarker {}

/// Marker for generate request values.
#[derive(Debug, Clone, Copy)]
pub struct GenerateRequestMarker;
impl ValueTypeMarker for GenerateRequestMarker {}

/// Marker for embeddings request values.
#[derive(Debug, Clone, Copy)]
pub struct EmbeddingsRequestMarker;
impl ValueTypeMarker for EmbeddingsRequestMarker {}

/// Marker for load model request values.
#[derive(Debug, Clone, Copy)]
pub struct LoadModelRequestMarker;
impl ValueTypeMarker for LoadModelRequestMarker {}

