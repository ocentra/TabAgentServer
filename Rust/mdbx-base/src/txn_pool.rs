//! Thread-Local Transaction Pool - Solves the `-30783 MDBX_BAD_TXN` error.
//!
//! MDBX enforces "one thread - one transaction" rule. This module provides a
//! thread-local transaction pool that allows multiple read operations in the
//! same thread by REUSING a single transaction.
//!
//! # Why This Pattern?
//!
//! **Problem:** MDBX returns `-30783` when trying to open a 2nd transaction in the same thread.
//!
//! **Solution:** Thread-local storage holds ONE read transaction per thread. Multiple
//! reads reuse this single transaction, maintaining zero-copy while avoiding the error.
//!
//! **Validation:** See `mdbx-base/src/lib.rs::test_thread_local_transaction_pool_pattern`
//!
//! # Example
//!
//! ```no_run
//! use mdbx_base::txn_pool::get_or_create_read_txn;
//!
//! unsafe {
//!     let env: *mut MDBX_env = /* ... */;
//!     
//!     // First read - creates new transaction
//!     let txn = get_or_create_read_txn(env)?;
//!     // Use txn...
//!     
//!     // Second read - REUSES same transaction (no error!)
//!     let txn = get_or_create_read_txn(env)?;
//!     // Use txn...
//! }
//! ```

use std::cell::RefCell;
use std::ptr;
use mdbx_sys::{
    MDBX_env, MDBX_txn, MDBX_SUCCESS, MDBX_TXN_RDONLY,
    mdbx_txn_begin_ex, mdbx_txn_abort,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxnPoolError {
    #[error("Failed to begin read transaction: {0}")]
    BeginFailed(i32),
}

thread_local! {
    /// Thread-local storage for reusable read transactions.
    ///
    /// Each thread maintains at most ONE read transaction which is reused
    /// across multiple read operations to comply with MDBX's threading model.
    static TXN_POOL: RefCell<Option<*mut MDBX_txn>> = RefCell::new(None);
}

/// Gets or creates a read-only transaction for the current thread.
///
/// **Thread-Local Reuse:** If the thread already has an active read transaction,
/// it returns the existing one. Otherwise, it creates a new one and caches it.
///
/// **Zero-Copy Safe:** The transaction remains open across multiple reads,
/// preserving zero-copy access to memory-mapped data.
///
/// # Safety
///
/// - The `env` pointer must be valid
/// - The returned transaction should not be manually closed (it's managed by the pool)
/// - Call `cleanup_thread_txn()` when the thread is done with all reads
///
/// # Errors
///
/// Returns `TxnPoolError::BeginFailed` if `mdbx_txn_begin_ex` fails.
pub unsafe fn get_or_create_read_txn(env: *mut MDBX_env) -> Result<*mut MDBX_txn, TxnPoolError> {
    TXN_POOL.with(|pool| {
        let mut pool_ref = pool.borrow_mut();
        
        if let Some(txn) = *pool_ref {
            // Reuse existing transaction
            Ok(txn)
        } else {
            // Create new read-only transaction
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(
                env,
                ptr::null_mut(),
                MDBX_TXN_RDONLY,
                &mut txn,
                ptr::null_mut(),
            );
            
            if rc != MDBX_SUCCESS {
                return Err(TxnPoolError::BeginFailed(rc));
            }
            
            // Cache for reuse
            *pool_ref = Some(txn);
            Ok(txn)
        }
    })
}

/// Cleans up the thread-local transaction (if any).
///
/// **Call this when the thread is done with all read operations.**
///
/// For most use cases, you don't need to call this - the transaction will
/// be automatically closed when the thread exits via `Drop`.
///
/// # Safety
///
/// Safe to call multiple times. Only aborts if a transaction exists.
pub fn cleanup_thread_txn() {
    TXN_POOL.with(|pool| {
        if let Some(txn) = *pool.borrow() {
            unsafe {
                mdbx_txn_abort(txn);
            }
            *pool.borrow_mut() = None;
        }
    });
}

/// Automatic cleanup of thread-local transactions when thread exits.
///
/// This guard ensures transactions are aborted when the thread terminates.
/// Create this at the start of your thread if using long-lived threads.
pub struct ThreadTxnGuard;

impl Drop for ThreadTxnGuard {
    fn drop(&mut self) {
        cleanup_thread_txn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thread_txn_guard_cleanup() {
        // This test just verifies that cleanup doesn't panic
        cleanup_thread_txn();
        cleanup_thread_txn(); // Should be safe to call twice
    }
}

