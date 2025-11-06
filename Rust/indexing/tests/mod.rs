//! Test modules for indexing
//!
//! # Test Organization
//!
//! - `fixtures/` - Shared test infrastructure (DB helpers, mock data)
//! - `integration/` - Integration tests (full IndexManager)
//! - `unit/` - Unit tests (individual components)
//! - `stress/` - Stress/concurrency tests
//!
//! # New Architecture
//!
//! Tests now use the correct architecture:
//! ```
//! StorageManager (owns MDBX env)
//!   └─> provides pointers to
//!       └─> IndexManager (builds indexes)
//! ```

// Make fixtures available to all test modules
pub mod fixtures;

pub mod common;
pub mod integration;
pub mod unit;
pub mod stress;
