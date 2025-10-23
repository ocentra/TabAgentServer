# Rust Architecture Guideline (RAG)

This Rust Architecture Guideline (RAG) provides a comprehensive set of rules and best practices for developing safe, performant, and maintainable Rust applications. AI systems (e.g., Cursor, Gemini) must internalize this spec, perform mental checklists after code blocks, and prioritize Rust's strengths to produce optimal code.

## Core Principles

1. **Safety First**: Leverage Rust's ownership, borrowing, and type system to eliminate entire classes of bugs (null pointer dereferences, data races, use-after-frees) at compile time. Safety is not optional.
2. **Performance by Design**: Apply zero-cost abstractions, prefer stack allocation over heap allocation, use efficient data structures, and leverage modern concurrency patterns and hardware capabilities (SIMD) where appropriate. Performance should be idiomatic, not an afterthought.
3. **Clarity and Maintainability**: Organize code by domain, document all public APIs thoroughly with runnable examples, write comprehensive tests, and favor readability. Code is read far more often than it is written.
4. **Ergonomic API Design**: Build APIs that are difficult to misuse. Leverage the type system to make impossible states unrepresentable and guide users toward correct usage.
5. **AI Collaboration**: Actively guide and correct AI-generated code to adhere to these principles. Focus on verifying ownership semantics, error handling robustness, lifetime correctness, and the justification for any `unsafe` code.
6. **Embrace Async for I/O-Bound Work**: Use `async/await` and a chosen runtime (like Tokio) for non-blocking concurrency to build highly scalable network services and applications.
7. **Manage Dependencies Wisely**: Minimize the dependency graph to reduce compile times and security surface. Audit all dependencies for vulnerabilities and excessive use of `unsafe` code.

When in doubt, consult the official Rust documentation, this guideline, and the Rustonomicon for unsafe code. AI must avoid pitfalls, justify deviations, and reference this guideline in comments if needed.

## Table of Contents

1. [Ownership Fundamentals](#ownership-fundamentals)
2. [Borrowing Rules](#borrowing-rules)
3. [Lifetimes](#lifetimes)
4. [Memory Safety](#memory-safety)
5. [Error Handling](#error-handling)
6. [Concurrency](#concurrency)
7. [Async Programming](#async-programming)
8. [Type System](#type-system)
9. [Traits and Generics](#traits-and-generics)
10. [Macros and Metaprogramming](#macros-and-metaprogramming)
11. [Unsafe Rust](#unsafe-rust)
12. [Unsafe Rust and FFI](#unsafe-rust-and-ffi)
13. [AI-Specific Guidelines](#ai-specific-guidelines)
14. [Common Anti-Patterns](#common-anti-patterns)
15. [Module Organization](#module-organization)
16. [Performance Optimizations](#performance-optimizations)
17. [Best Practices](#best-practices)
18. [Testing Strategies](#testing-strategies)
19. [Testing Principles](#testing-principles)
20. [Dependency Management](#dependency-management)
21. [Security Best Practices](#security-best-practices)
21. [Build, CI, and Tooling](#build-ci-and-tooling)
22. [Migration and Maintenance](#migration-and-maintenance)
23. [Checklist](#checklist)
24. [Expected Benefits](#expected-benefits)

## Ownership Fundamentals

### Rule 1.1: Each Value Has Exactly One Owner

- Every value has a single variable that is its owner. When the owner goes out of scope, the value is dropped, and its memory is freed. This is the core of Rust's RAII (Resource Acquisition Is Initialization) pattern.
- Ownership is transferred on assignment, function calls, or returns. This is a "move."
- **AI Note**: Critically trace ownership paths. In loops, do not move a value that is needed in a subsequent iteration. The AI should prefer borrowing within loops or iterating over references.

```rust
fn process(data: Vec<i32>) {
    // data owned here
}
let numbers = vec![1, 2, 3];
process(numbers); // Moved
```

### Rule 1.2: Clone When Ownership is Required, Not by Default

- Prefer passing references (`&T`, `&mut T`) to avoid expensive deep copies.
- Implement the `Copy` trait for small, plain-old-data (POD) types that can be duplicated with a cheap bitwise copy. `Copy` is a superset of `Clone`.
- Use `.clone()` only when you explicitly need to duplicate data and create a new owner. Profile code in hot paths before liberally adding clones.
- **AI Note**: Justify every single `.clone()` call. Ask: "Can I use a borrow? Can I use `Arc` for shared ownership? Am I cloning inside a hot loop?"

```rust
fn len(s: &str) -> usize { s.len() }
let s = String::from("hello");
let length = len(&s); // Borrow, no clone
```

### Rule 1.3: Prefer Move Semantics for Large Data

- Moving large data structures (like a `Vec<T>`) is extremely cheap, as it only involves copying a few stack-allocated pointers and metadata, not the heap data itself.
- Design functions to take ownership of large data structures when they need to consume or transform them. Use traits like `IntoIterator` to be generic over owned values and references.

```rust
fn process_large(data: Vec<LargeStruct>) {
    // Process data
}
let data = create_large_data();
process_large_data(data); // Efficient move
```

## Borrowing Rules

### Rule 2.1: Either Multiple Immutable Borrows (`&T`) OR One Mutable Borrow (`&mut T`)

- This rule is checked at compile time and prevents data races. You cannot have both at the same time in the same scope.
- A mutable borrow is exclusive. While it exists, no other borrows of the object are allowed.
- **Note on Non-Lexical Lifetimes (NLL)**: The borrow checker is smart. A borrow ends when the reference is last used, not necessarily at the end of the lexical scope (`}`).

```rust
let mut x = 5;
let y = &mut x; // Mutable borrow starts
*y += 1;
// Mutable borrow y is last used here, so it ends.
println!("The value of x is: {}", x); // Now we can immutably borrow x again.
```

### Rule 2.2: Explicit Lifetimes Are Required When Ambiguous

- The compiler can infer lifetimes in many situations (lifetime elision), but you must be explicit when reference relationships are not obvious.
- **Rule of thumb**: If a function takes multiple references as input and returns one, you must annotate lifetimes to tell the compiler which input reference the output reference is tied to.
- Use `'static` sparingly. It means the reference is valid for the entire duration of the program. It's common for string literals or global constants.
- **AI Note**: When generating a function with reference inputs and outputs, double-check if lifetime annotations are required. If the compiler complains, add the necessary `'a`, `'b`, etc. annotations.

```rust
fn process_slice(slice: &[i32]) {
    // Process slice
}
let vec = vec![1, 2, 3];
let first = &vec[0];
process_slice(&vec);
println!("{}", first);
```

### Rule 2.3: Use Struct Lifetime Annotations for Stored References

- If a struct holds a reference to data it does not own, its definition must be annotated with a lifetime parameter. This ensures that no instance of the struct can outlive the data it refers to.

```rust
// This Context holds a reference to some string data.
// The 'a ensures the Context cannot outlive the data.
struct Context<'a> {
    data: &'a str,
}
```

## Lifetimes

### Rule 3.1: Explicit Lifetimes When References Outlive Scope

- Use 'static sparingly

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

### Rule 3.2: Avoid Lifetime Elision Ambiguity

- Annotate if >1 input ref

```rust
fn get_first<'a>(s: &'a str) -> &'a str {
    &s[0..1]
}
```

### Rule 3.3: Prefer Struct Lifetime Annotations for Stored References

```rust
struct Context<'a> {
    data: &'a str,
}
```

### Rule 3.4: Use Higher-Ranked Trait Bounds (HRTB) for Flexible Lifetimes

- For closures or traits with refs

```rust
fn for_any_lifetime<F>(f: F) where F: for<'a> FnOnce(&'a str) {}
```

## Memory Safety

### Rule 4.1: Eliminate Null Pointers with `Option<T>`

- Rust does not have null pointers. For values that can be absent, use the `Option<T>` enum, which can be `Some(T)` or `None`.
- This forces you to handle the `None` case at compile time, preventing null pointer dereferences.

```rust
fn find_item(haystack: &[i32], needle: i32) -> Option<usize> {
    haystack.iter().position(|&x| x == needle)
}
```

### Rule 4.2: Leverage RAII for All Resource Management

- The `Drop` trait is Rust's equivalent of a destructor. Implement `Drop` for any type that needs to perform cleanup (e.g., closing a file, releasing a lock, closing a network connection). The compiler will automatically insert calls to `drop` when the value goes out of scope.

```rust
fn as_string() -> String {
    String::from("hello")
}
```

### Rule 4.3: Use the Right Smart Pointer for the Job

- `Box<T>`: For heap allocation of a single value. Provides unique ownership.
- `Rc<T>`: For **single-threaded** shared ownership. Uses reference counting.
- `Arc<T>`: For **thread-safe** shared ownership. Uses **A**tomic **R**eference **C**ounting. It is the thread-safe equivalent of `Rc<T>`.
- `RefCell<T>`: For enforcing borrowing rules at **runtime** instead of compile time (interior mutability) in a **single-threaded** context. A panic will occur if borrowing rules are violated.
- `Mutex<T>` / `RwLock<T>`: For **thread-safe** interior mutability. `Mutex` provides exclusive access, while `RwLock` allows multiple readers or one writer, making it more performant for read-heavy workloads.

```rust
let shared = Arc::new(data);
```

## Error Handling

### Rule 5.1: Use `Result<T, E>` for All Recoverable Errors

- If an operation can fail but the failure is expected and handleable, the function must return `Result<T, E>`.
- Use the `?` operator for clean, concise error propagation. It unwraps a `Result` or returns the `Err` variant from the function early.
- **AI Note**: Never generate code that calls `.unwrap()` or `.expect()` on `Result` or `Option` types in production logic. These should only be used in tests, examples, or in cases where a panic is the absolutely correct and desired outcome (e.g., a critical, unrecoverable initialization failure).

```rust
fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
```

### Rule 5.2: Use Specific, Custom Error Types

- For libraries, define custom error enums that represent all possible failure modes.
- Use the `thiserror` crate to easily derive `std::error::Error` and create rich, expressive error types with source information. This is ideal for libraries.
- For applications, use the `anyhow` crate for simple, flexible error handling with context and backtraces. It is excellent for the top layers of an application (`main.rs`, request handlers, etc.).

```rust
use thiserror::Error;
#[derive(Error, Debug)]
enum AppError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}
```

### Rule 5.3: Differentiate Between Recoverable Errors (`Result`) and Unrecoverable Errors (`panic!`)

- `panic!` should be reserved for truly exceptional circumstances that indicate a bug in the program (e.g., a violated invariant, an out-of-bounds access on an index you thought was safe). A panic unwinds the stack and crashes the thread.

```rust
let value = map.get("key").ok_or_else(|| AppError::KeyMissing("key".to_string()))?;
```

### Rule 5.4: Implement Error Tracing

- Use eyre for stack traces

```rust
fn func() -> eyre::Result<()> {
    // Implementation
    Ok(())
}
```

## Concurrency

### Rule 6.1: Use Message Passing for Concurrency

- Prefer channels (mpsc)

```rust
use std::sync::mpsc;
use std::thread;
let (tx, rx) = mpsc::channel();
thread::spawn(move || {
    tx.send("Hello from thread").unwrap();
});
println!("{}", rx.recv().unwrap());
```

### Rule 6.2: Use Mutexes for Shared State

- Prefer RwLock for read-heavy

```rust
use std::sync::{Arc, Mutex, RwLock};
let counter = Arc::new(Mutex::new(0));
let data = Arc::new(RwLock::new(0));
```

### Rule 6.3: Avoid Data Races

- Use Sync bounds

```rust
fn process<T: Sync>(data: &T) {
    // Thread-safe
}
```

## Async Programming

### Rule 7.1: Use async/await for Non-Blocking I/O

- Prefer Tokio or async-std

```rust
async fn fetch() -> Result<String, Error> {
    reqwest::get("url").await?.text().await
}
```

### Rule 7.2: Avoid Blocking in Async Contexts

- Use tokio::spawn for CPU-bound

```rust
use tokio::task;
task::spawn_blocking(|| heavy_computation()).await?;
```

### Rule 7.3: Manage Cancellation with Futures

- Use select! for racing

```rust
use tokio::select;
select! {
    _ = task1() => {},
    _ = task2() => {}
}
```

## Type System

### Rule 8.1: Leverage the Newtype Pattern for Domain-Specific Types

- Wrap primitive types (like `i32` or `String`) in a tuple struct to create a new, distinct type. This allows the compiler to enforce domain-specific invariants.

```rust
struct UserId(i32);
struct ProductId(String);

// This function cannot accidentally be called with a ProductId.
fn get_user_by_id(id: UserId) { /* ... */ }
```

### Rule 8.2: Make Impossible States Unrepresentable

- Use enums and struct layouts to model your domain in a way that invalid states cannot even be created.

```rust
// BAD: Boolean blindness and invalid states are possible (e.g., connected AND error_message is Some)
struct Connection {
    is_connected: bool,
    is_connecting: bool,
    since: Option<std::time::Instant>,
    error_message: Option<String>,
}

// GOOD: Each state is distinct and holds only the data relevant to it.
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { since: std::time::Instant },
    Failed { error: String },
}
```

### Rule 8.3: Use Phantom Types for Type Safety

```rust
use std::marker::PhantomData;
struct Degrees<T> {
    value: f64,
    phantom: PhantomData<T>,
}
struct Celsius;
struct Fahrenheit;
impl Degrees<Celsius> {
    fn to_fahrenheit(&self) -> Degrees<Fahrenheit> {
        Degrees {
            value: self.value * 9.0/5.0 + 32.0,
            phantom: PhantomData,
        }
    }
}
```

### Rule 8.4: Use Associated Types in Traits

- For related types

```rust
trait Graph {
    type Node;
    fn nodes(&self) -> Vec<Self::Node>;
}
```

## Traits and Generics

### Rule 9.1: Use Traits for Polymorphism

- Define behavior via traits

```rust
trait Drawable {
    fn draw(&self);
}
struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }
impl Drawable for Circle {
    fn draw(&self) { /* ... */ }
}
impl Drawable for Rectangle {
    fn draw(&self) { /* ... */ }
}
fn draw_all<T: Drawable>(shapes: &[T]) {
    for shape in shapes {
        shape.draw();
    }
}
```

### Rule 9.2: Prefer Trait Objects Over Enums for Open Sets

```rust
fn draw_shapes(shapes: &Vec<Box<dyn Drawable>>) {
    for shape in shapes {
        shape.draw();
    }
}
```

### Rule 9.3: Implement Standard Traits Thoughtfully

- Use derive where possible

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}
```

## Macros and Metaprogramming

### Rule 10.1: Prefer Functions Over Macros When Possible

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Rule 10.2: Use `macro_rules!` for Declarative Macros

```rust
macro_rules! create_vec {
    ($($x:expr),*) => {
        {
            let mut temp_vec = Vec::new();
            $(temp_vec.push($x);)*
            temp_vec
        }
    };
}
```

### Rule 10.3: Use Procedural Macros for Code Generation

```rust
use proc_macro::TokenStream;
#[proc_macro_derive(Builder)]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    // Implementation
}
```

## Unsafe Rust

### Rule 11.1: Minimize and Isolate `unsafe` Code

- The `unsafe` keyword does not turn off the borrow checker; it only allows five extra capabilities (dereferencing raw pointers, calling unsafe functions, etc.).
- You, the programmer, are responsible for manually upholding Rust's safety invariants within an `unsafe` block.
- Wrap `unsafe` blocks in a safe abstraction. The goal is to have a safe public API, even if the internal implementation requires a small amount of `unsafe` for performance or interoperability.

```rust
struct SafeVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}
impl<T> SafeVec<T> {
    pub fn new() -> Self {
        SafeVec {
            ptr: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }
    pub fn push(&mut self, item: T) {
        // Safe implementation
    }
}
```

### Rule 11.2: Document Safety Invariants Thoroughly

- Every `unsafe` block must be accompanied by a `// SAFETY:` comment that explains *why* the code is safe and what invariants must be upheld by the surrounding code for it to be correct.
- **AI Note**: An AI must never generate `unsafe` code without a corresponding `// SAFETY:` comment. It should also be challenged to provide a safe alternative first.

```rust
/// # Safety
/// 1. `ptr` is non-null and aligned
/// 2. `ptr` points to valid memory
/// 3. Memory not mutated elsewhere
unsafe fn read_memory(ptr: *const u8, len: usize) -> Vec<u8> {
    // Implementation
}
```

### Rule 11.3: Validate Unsafe Code with Tests

```rust
#[test]
fn test_safe_abstraction() {
    let mut vec = SafeVec::new();
    vec.push(42);
    assert_eq!(vec.get(0), Some(&42));
}
```

## Unsafe Rust and FFI

### Rule 11.3: Adhere to FFI (Foreign Function Interface) Best Practices

- Use `#[repr(C)]` on structs passed across the FFI boundary to ensure a stable memory layout.
- Use types from `std::os::raw` (like `c_char`) or the `libc` crate for C-compatible types.
- **Never panic across an FFI boundary.** This is undefined behavior. Wrap FFI entry points in `std::panic::catch_unwind` and return an error code instead.

```rust
#[repr(C)]
pub struct CStruct {
    pub field: i32,
}

#[no_mangle]
pub extern "C" fn safe_ffi_function() -> i32 {
    std::panic::catch_unwind(|| {
        // Safe Rust code here
        42
    }).unwrap_or(-1) // Return error code on panic
}
```

## AI-Specific Guidelines

### Rule 12.1: Always Verify Ownership and Borrowing

- **Mental Checklist**:
  - Is `.clone()` used where a borrow `&` would suffice? This is the most common AI error.
  - Is a value being moved inside a loop, causing a "use of moved value" error on the second iteration?
  - Could `Arc` be used for more efficient shared ownership instead of deep cloning?
- **Action**: Instruct the AI to refactor to use borrows, iterators over references, or appropriate smart pointers.

```rust
fn process(data: Vec<i32>) -> Vec<i32> {
    let processed = data.iter().map(|x| x * 2).collect();
    processed
}
```

### Rule 12.2: Enforce Robust Error Handling

- **Mental Checklist**:
  - Is `.unwrap()` or `.expect()` used on a `Result` or `Option`?
- **Action**: Immediately instruct the AI to replace it with proper error handling (`match`, `if let`, `?`). Provide the context for a proper error type (`thiserror` enum or `anyhow::Result`).

```rust
fn first_and_last(s: &str) -> (&str, &str) {
    let first = &s[0..1];
    let last = &s[s.len()-1..];
    (first, last)
}
```

### Rule 12.3: Scrutinize `unsafe` Code Blocks

- **Mental Checklist**:
  - Is this `unsafe` block absolutely necessary? Can it be achieved with safe Rust?
  - Is there a `// SAFETY:` comment explaining the invariants?
- **Action**: Challenge the AI to provide a safe alternative. If `unsafe` is required, ensure it is documented correctly and isolated in the smallest possible scope.

```rust
fn get_unchecked(vec: &Vec<i32>, index: usize) -> Option<&i32> {
    if index < vec.len() {
        Some(unsafe { vec.get_unchecked(index) })
    } else {
        None
    }
}
```

### Rule 12.4: Demand Idiomatic Rust

- **Mental Checklist**:
  - Is the AI using C-style `for i in 0..len { vec[i] }` loops instead of iterators?
  - Is it using boolean flags instead of enums for state?
  - Is it using `String` in function arguments where `&str` would be more flexible?
- **Action**: Instruct the AI to refactor using iterators (`.iter()`, `.map()`, `.filter()`), enums, and appropriate string types (`&str`, `Path`, `OsStr`).

```rust
fn find<T: PartialEq>(vec: &Vec<T>, item: &T) -> Option<usize> {
    vec.iter().position(|x| x == item)
}
```

### Rule 12.5: Generate Documentation and Tests with Code

- **Action**: Always require the AI to generate `rustdoc` comments (`///`) for all public functions, structs, and enums. These comments should include a brief explanation, sections for `# Examples`, `# Panics`, and `# Errors`.
- **Action**: For any non-trivial function, instruct the AI to also generate a `#[cfg(test)] mod tests { ... }` module with unit tests covering both success and failure cases.
- **Action**: Instruct the AI to follow Test-Driven Development (TDD) principles, writing tests that validate real functionality rather than just trying to make tests pass.
- **Action**: Require the AI to avoid stubs and mocks except when absolutely necessary, preferring real implementations and real data in tests.

```rust
/// Adds two numbers together.
/// 
/// # Examples
/// ```
/// assert_eq!(add(2, 3), 5);
/// ```
fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() { 
        assert_eq!(add(2, 3), 5); 
    }
}
```

## Common Anti-Patterns

### Rule 13.1: Avoid `RefCell` Overuse

```rust
struct Counter {
    count: u32,
}
impl Counter {
    fn increment(&mut self) {
        self.count += 1;
    }
}
```

### Rule 13.2: Don't Fight the Borrow Checker

- Redesign code to align with ownership

### Rule 13.3: Avoid Stringly-Typed APIs

```rust
enum Color { Red, Green, Blue }
fn set_color(color: Color) { /* ... */ }
```

### Rule 13.4: Prevent Iterator Invalidation

```rust
let mut vec = vec![1, 2, 3, 4];
vec.retain(|&x| x % 2 != 0);
```

### Rule 13.5: Avoid String Literals for Domain-Specific Values

- String literals are prone to typos and make refactoring difficult. They provide no type safety and can lead to runtime errors.
- Use enums, constants, or newtype patterns for domain-specific string values.
- This approach prevents accidental misspellings, enables compile-time checking, and improves code documentation.

```rust
// BAD: Stringly-typed API with literals
fn set_status(status: &str) -> Result<(), Error> {
    match status {
        "active" => { /* ... */ },
        "inactive" => { /* ... */ },
        "pending" => { /* ... */ },
        _ => return Err(Error::InvalidStatus),
    }
    Ok(())
}

// GOOD: Type-safe enum
#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    Active,
    Inactive,
    Pending,
}

fn set_status(status: Status) -> Result<(), Error> {
    match status {
        Status::Active => { /* ... */ },
        Status::Inactive => { /* ... */ },
        Status::Pending => { /* ... */ },
    }
    Ok(())
}

// ALSO GOOD: Constants for fixed string values
const STATUS_ACTIVE: &str = "active";
const STATUS_INACTIVE: &str = "inactive";
const STATUS_PENDING: &str = "pending";

// BEST: Newtype pattern with validation
#[derive(Debug, Clone, PartialEq)]
struct Status(String);

impl Status {
    pub fn active() -> Self { Status(STATUS_ACTIVE.to_string()) }
    pub fn inactive() -> Self { Status(STATUS_INACTIVE.to_string()) }
    pub fn pending() -> Self { Status(STATUS_PENDING.to_string()) }
    
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            STATUS_ACTIVE => Ok(Status::active()),
            STATUS_INACTIVE => Ok(Status::inactive()),
            STATUS_PENDING => Ok(Status::pending()),
            _ => Err(Error::InvalidStatus),
        }
    }
}
```

## Module Organization

### Rule 14.1: Follow Domain-Driven Structure

```rust
src/
├── core/
│   ├── error.rs
│   └── mod.rs
├── media/
│   ├── audio/
│   ├── video/
│   └── mod.rs
├── timeline/
│   ├── models.rs
│   └── operations.rs
└── lib.rs
```

### Rule 14.2: Separate Concerns into Modules

```rust
mod project {
    pub mod settings;
    pub mod resolution;
}
mod media {
    pub mod info;
    pub mod thumbnail;
}
```

### Rule 14.3: Use Clear Module Boundaries

```rust
pub mod api {
    pub fn public_function() { /* ... */ }
    fn internal_helper() { /* ... */ }
}
mod internal {
    pub(crate) fn crate_visible() { /* ... */ }
    fn private() { /* ... */ }
}
```

## Performance Optimizations

### Rule 15.1: Implement Zero-Cost Abstractions

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}
impl Resolution {
#[inline]
pub fn aspect_ratio(&self) -> f64 {
    self.width as f64 / self.height as f64
    }
}
```

### Rule 15.2: Optimize String Handling

```rust
pub fn process_path(path: &str) -> Result<(), Error> {
    // Process without cloning
    Ok(())
}
use std::borrow::Cow;
pub struct Metadata {
pub name: Cow<'static, str>,
}
```

### Rule 15.3: Use Cache-Friendly Data Structures

```rust
use std::collections::BTreeMap;
use smallvec::SmallVec;
pub struct Timeline {
    pub clips: SmallVec<[ClipData; 8]>,
    pub markers: BTreeMap<Time, Marker>,
}
```

### Rule 15.4: Leverage SIMD for Media Processing

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
pub fn process_audio(samples: &mut [f32]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx") {
            return unsafe { process_audio_avx(samples) };
        }
    }
    // Fallback
}
```

### Rule 15.5: Implement Memory Pools for Frequent Allocations

```rust
pub struct ClipPool {
    pool: Vec<ClipData>,
}
impl ClipPool {
    pub fn get(&mut self) -> ClipData {
        self.pool.pop().unwrap_or_default()
    }
    pub fn return_clip(&mut self, clip: ClipData) {
        self.pool.push(clip);
    }
}
```

## Best Practices

### Rule 16.1: Follow Rust Naming Conventions

```rust
pub struct VideoClip {
    pub duration: Duration,
    pub frame_rate: FrameRate,
}
pub fn calculate_timeline(timeline: &Timeline) -> Result<Duration, Error> {
    Ok(Duration::new(0, 0))
}
pub const MAX_RESOLUTION: Resolution = Resolution {
    width: 3840,
    height: 2160,
};
```

### Rule 16.2: Document Public APIs

```rust
/// Represents a video clip.
/// 
/// # Examples
/// ```
/// let clip = VideoClip::new(Duration::from_secs(10), FrameRate::fps(30));
/// assert_eq!(clip.duration(), Duration::from_secs(10));
/// ```
pub struct VideoClip {
    // Fields
}
```

### Rule 16.3: Write Integration Tests and Benchmarks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clip_creation() {
        let clip = VideoClip::new(Duration::from_secs(10), FrameRate::fps(30));
        assert_eq!(clip.duration(), Duration::from_secs(10));
    }
}
#[bench]
fn bench_timeline_processing(b: &mut test::Bencher) {
    let timeline = create_test_timeline();
    b.iter(|| process_timeline(&timeline));
}
```

### Rule 16.4: Ensure Memory Safety

```rust
pub fn safe_vector_access(vec: &Vec<i32>, index: usize) -> Option<&i32> {
    vec.get(index)
}
```

### Rule 16.5: Use Linting and Formatting Tools

- Run cargo clippy, cargo fmt

```rust
// In CI
// cargo clippy -- -D warnings
// cargo fmt
```

## Testing Strategies

### Rule 17.1: Write Unit and Integration Tests

- Cover happy/error paths

```rust
#[test]
fn test() {
    assert_eq!(func(), expected);
}
```

### Rule 17.2: Use Property-Based Testing

- With proptest

```rust
proptest! {
    #[test]
    fn prop(input: i32) {
        // Test properties
    }
}
```

### Rule 17.3: Include Fuzz Testing for Inputs

- Use cargo fuzz

```rust
// cargo fuzz init
```

## Testing Principles

### Rule 17.4: Every Code Addition Requires Tests

- If there is code, there must be corresponding tests.
- All functionality, whether critical or non-critical, must have regression tests.
- Tests must be written before or alongside the code, following Test-Driven Development (TDD) principles.

### Rule 17.5: Avoid Stubs and Mocks Except When Absolutely Necessary

- Stubs and mocks should only be used in rare occasions and for temporary reasons.
- Prefer real implementations and real data in tests whenever possible.
- If a test requires a stub or mock, clearly document why it's necessary.

### Rule 17.6: Tests Must Validate Real Functionality

- Do not retrofit tests just to make them pass.
- If a test fails, it should fail for a valid reason that indicates a real issue.
- The core idea of testing is not to simply pass tests, but to validate functionality so we can fix actual code.
- A failing test is better than a test with stub or mock data.
- Always test against real data as much as possible.
- Follow TDD and industry standards for test quality.

## Dependency Management

### Rule 18.1: Minimize Dependencies

- Audit with cargo audit

```rust
// cargo.toml
[dependencies]
# Minimal set
```

### Rule 18.2: Pin Versions

- Use Cargo.lock

```rust
// Update Cargo.lock
// cargo update
```

### Rule 18.3: Prefer No-Std When Possible

- For embedded systems

```rust
#![no_std]
```

## Security Best Practices

### Rule 19.1: Sanitize Inputs

- Use validators

```rust
fn validate_input(input: &str) -> Result<(), Error> {
    if input.contains("<") { Err(Error::InvalidInput) } else { Ok(()) }
}
```

### Rule 19.2: Avoid Deserialization Vulnerabilities

- Use serde safely

```rust
use serde::Deserialize;
#[derive(Deserialize)]
struct SafeData {
    #[serde(deserialize_with = "validate")]
    field: String,
}
```

### Rule 19.3: Use Secure Randomness

- rand with crypto backends

```rust
use rand::rngs::OsRng;
let value = OsRng.gen::<u32>();
```

## Build, CI, and Tooling

### Rule 18.1: Enforce Code Quality with CI

- Your CI pipeline must, at a minimum, run:
  1. `cargo fmt --check`: Ensures all code conforms to the standard Rust style.
  2. `cargo clippy -- -D warnings`: Runs the linter and fails the build on any warnings. This catches common mistakes and anti-patterns.
  3. `cargo test`: Runs all unit and integration tests.

### Rule 18.2: Audit Dependencies

- Regularly run `cargo audit` to check for security vulnerabilities in your dependencies.
- Use `cargo-deny` or `cargo-geiger` in CI to check for disallowed licenses or high concentrations of `unsafe` code in your dependency tree.

### Rule 18.3: Use `rust-toolchain.toml`

- Pin the exact Rust toolchain version for your project by adding a `rust-toolchain.toml` file to your repository root. This ensures all developers and the CI system use the exact same compiler version, leading to reproducible builds.

```toml
[toolchain]
channel = "1.75.0"
components = ["rustfmt", "clippy"]
```

## Migration and Maintenance

### Rule 20.1: Plan Incremental Migration

```rust
// Phase 1: Create new module structure
// Phase 2: Move domain logic
// Phase 3: Optimize paths
// Phase 4: Update APIs
```

### Rule 20.2: Measure Performance Improvements

```rust
#[bench]
fn bench_old(b: &mut test::Bencher) {
    b.iter(|| old_process(&data));
}
#[bench]
fn bench_new(b: &mut test::Bencher) {
    b.iter(|| new_process(&data));
}
```

### Rule 20.3: Maintain a Rust Best Practices Checklist

```rust
// Checklist
// - [ ] All public APIs documented
// - [ ] No unwrap() in production
```

## Checklist

### Rust Best Practices Checklist (For AI and Human Review)

- [ ] **No `.unwrap()`/`.expect()` in production code.**
- [ ] **Error handling uses `Result<T, E>` and `?`.**
- [ ] **Domain-driven module structure is used.**
- [ ] **Public APIs are documented with `///` and runnable examples.**
- [ ] **Clones are intentional and justified; borrows are preferred.**
- [ ] **`&str` is used for function parameters instead of `String` where possible.**
- [ ] **Type system is used to prevent invalid states (e.g., enums over booleans).**
- [ ] **`unsafe` blocks are minimal, isolated, and have `// SAFETY:` comments.**
- [ ] **CI checks for formatting (`fmt`), linting (`clippy`), and tests.**
- [ ] **Dependencies are audited (`cargo audit`).**
- [ ] **Iterators are used instead of manual C-style index loops.**
- [ ] **Appropriate smart pointers (`Box`, `Arc`, `Rc`) are used.**
- [ ] **Benchmarks exist for performance-critical code.**
- [ ] **`[inline]` is used judiciously for small, hot functions.**
- [ ] **Follows standard Rust naming conventions (snake_case for functions/variables, PascalCase for types).**
- [ ] **Domain-specific string values use enums or constants instead of string literals.**
- [ ] **Every code addition has corresponding tests.**
- [ ] **All critical and non-critical functionality has regression tests.**
- [ ] **Stubs and mocks are avoided except when absolutely necessary.**
- [ ] **Tests validate real functionality, not just pass for the sake of passing.**
- [ ] **Failing tests indicate real issues that need to be fixed.**
- [ ] **Tests use real data whenever possible instead of stub/mock data.**
- [ ] **Follows Test-Driven Development (TDD) principles.**


1. **Ownership Fundamentals** (Rules 1.1-1.3)
   - [ ] Each value has exactly one owner
   - [ ] Clone only when ownership is required
   - [ ] Prefer move semantics for large data

2. **Borrowing Rules** (Rules 2.1-2.3)
   - [ ] Either multiple immutable borrows OR one mutable borrow
   - [ ] Explicit lifetimes when ambiguous
   - [ ] Struct lifetime annotations for stored references

3. **Lifetimes** (Rules 3.1-3.4)
   - [ ] Explicit lifetimes when references outlive scope
   - [ ] Avoid lifetime elision ambiguity
   - [ ] Struct lifetime annotations for stored references
   - [ ] Higher-ranked trait bounds for flexible lifetimes

4. **Memory Safety** (Rules 4.1-4.3)
   - [ ] Eliminate null pointers with `Option<T>`
   - [ ] Leverage RAII for resource management
   - [ ] Use appropriate smart pointers

5. **Error Handling** (Rules 5.1-5.4)
   - [ ] Use `Result<T, E>` for recoverable errors
   - [ ] Use specific, custom error types
   - [ ] Differentiate recoverable vs unrecoverable errors
   - [ ] Implement error tracing

6. **Concurrency** (Rules 6.1-6.3)
   - [ ] Use message passing
   - [ ] Use mutexes for shared state
   - [ ] Avoid data races

7. **Async Programming** (Rules 7.1-7.3)
   - [ ] Use async/await for I/O
   - [ ] Avoid blocking in async contexts
   - [ ] Manage cancellation

8. **Type System** (Rules 8.1-8.4)
   - [ ] Leverage newtype pattern
   - [ ] Make impossible states unrepresentable
   - [ ] Use phantom types
   - [ ] Use associated types

9. **Traits and Generics** (Rules 9.1-9.3)
   - [ ] Use traits for polymorphism
   - [ ] Prefer trait objects over enums
   - [ ] Implement standard traits

10. **Macros and Metaprogramming** (Rules 10.1-10.3)
    - [ ] Prefer functions over macros
    - [ ] Use `macro_rules!` for declarative macros
    - [ ] Use procedural macros

11. **Unsafe Rust** (Rules 11.1-11.3)
    - [ ] Minimize and isolate unsafe code
    - [ ] Document safety invariants
    - [ ] Validate with tests

12. **Unsafe Rust and FFI** (Rule 11.3)
    - [ ] Follow FFI best practices

13. **AI-Specific Guidelines** (Rules 12.1-12.5)
    - [ ] Verify ownership and borrowing
    - [ ] Enforce robust error handling
    - [ ] Scrutinize unsafe code
    - [ ] Demand idiomatic Rust
    - [ ] Generate documentation and tests

14. **Common Anti-Patterns** (Rules 13.1-13.5)
    - [ ] Avoid RefCell overuse
    - [ ] Don't fight the borrow checker
    - [ ] Avoid stringly-typed APIs
    - [ ] Prevent iterator invalidation
    - [ ] Avoid string literals for domain values

15. **Module Organization** (Rules 14.1-14.3)
    - [ ] Follow domain-driven structure
    - [ ] Separate concerns
    - [ ] Clear module boundaries

16. **Performance Optimizations** (Rules 15.1-15.5)
    - [ ] Zero-cost abstractions
    - [ ] Optimize string handling
    - [ ] Cache-friendly data structures
    - [ ] SIMD acceleration
    - [ ] Memory pools

17. **Best Practices** (Rules 16.1-16.5)
    - [ ] Naming conventions
    - [ ] Document public APIs
    - [ ] Integration tests and benchmarks
    - [ ] Memory safety
    - [ ] Linting and formatting

18. **Testing Strategies** (Rules 17.1-17.3)
    - [ ] Unit and integration tests
    - [ ] Property-based testing
    - [ ] Fuzz testing

19. **Testing Principles** (Rules 17.4-17.6)
    - [ ] Every code addition requires tests
    - [ ] Avoid stubs and mocks
    - [ ] Validate real functionality

20. **Dependency Management** (Rules 18.1-18.3)
    - [ ] Minimize dependencies
    - [ ] Pin versions
    - [ ] Prefer no-std when possible

21. **Security Best Practices** (Rules 19.1-19.3)
    - [ ] Sanitize inputs
    - [ ] Avoid deserialization vulnerabilities
    - [ ] Use secure randomness

22. **Build, CI, and Tooling** (Rules 18.1-18.3)
    - [ ] Enforce code quality with CI
    - [ ] Audit dependencies
    - [ ] Use rust-toolchain.toml


## Expected Benefits

### Developer Experience

- ✅ Easy to find related code
- ✅ Clear domain boundaries
- ✅ Follows Rust conventions
- ✅ Better IDE support
- ✅ Comprehensive documentation
- ✅ AI-friendly patterns

### Performance

- ✅ Fewer allocations
- ✅ Better cache locality
- ✅ SIMD acceleration
- ✅ Predictable performance
- ✅ Efficient memory usage
- ✅ Async efficiency

### Maintainability

- ✅ Clear separation of concerns
- ✅ Easy to test individual domains
- ✅ Scalable architecture
- ✅ Future-proof design
- ✅ Incremental migration path
- ✅ Automated linting

### Safety

- ✅ Compile-time error prevention
- ✅ Memory safety guarantees
- ✅ Thread-safe concurrency
- ✅ Minimal unsafe code
- ✅ Comprehensive validation
- ✅ Secure dependencies
