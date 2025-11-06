//! Transaction Helpers - High-level wrappers for MDBX transactions.
//!
//! Eliminates boilerplate for common transaction patterns:
//! - `with_read_txn()` - Execute read operation with automatic abort
//! - `with_write_txn()` - Execute write operation with automatic commit/abort
//! - `open_dbi()` - Open named database with error handling

use std::ffi::CString;
use std::ptr;
use mdbx_sys::{
    MDBX_env, MDBX_txn, MDBX_dbi, MDBX_SUCCESS,
    MDBX_TXN_RDONLY, MDBX_CREATE,
    mdbx_txn_begin_ex, mdbx_txn_commit_ex, mdbx_txn_abort,
    mdbx_dbi_open,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxnError {
    #[error("Failed to begin transaction: {0}")]
    BeginFailed(i32),
    
    #[error("Failed to commit transaction: {0}")]
    CommitFailed(i32),
    
    #[error("Failed to open DBI '{name}': {code}")]
    DbiOpenFailed { name: String, code: i32 },
    
    #[error("Invalid database name: {0}")]
    InvalidName(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Executes a read-only operation within a transaction.
///
/// The transaction is automatically aborted after the operation completes.
///
/// # Example
///
/// ```no_run
/// use mdbx_base::txn_helpers::with_read_txn;
///
/// unsafe {
///     let env: *mut MDBX_env = /* ... */;
///     let result = with_read_txn(env, |txn| {
///         // Read operations using txn...
///         Ok(42)
///     })?;
/// }
/// ```
pub unsafe fn with_read_txn<F, T>(env: *mut MDBX_env, operation: F) -> Result<T, TxnError>
where
    F: FnOnce(*mut MDBX_txn) -> Result<T, TxnError>,
{
    let mut txn: *mut MDBX_txn = ptr::null_mut();
    let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), MDBX_TXN_RDONLY, &mut txn, ptr::null_mut());
    
    if rc != MDBX_SUCCESS {
        return Err(TxnError::BeginFailed(rc));
    }
    
    let result = operation(txn);
    
    // Always abort read transactions (they don't modify data)
    mdbx_txn_abort(txn);
    
    result
}

/// Executes a read-write operation within a transaction.
///
/// The transaction is automatically committed on success or aborted on error.
///
/// # Example
///
/// ```no_run
/// use mdbx_base::txn_helpers::with_write_txn;
///
/// unsafe {
///     let env: *mut MDBX_env = /* ... */;
///     with_write_txn(env, |txn| {
///         // Write operations using txn...
///         Ok(())
///     })?;
/// }
/// ```
pub unsafe fn with_write_txn<F, T>(env: *mut MDBX_env, operation: F) -> Result<T, TxnError>
where
    F: FnOnce(*mut MDBX_txn) -> Result<T, TxnError>,
{
    let mut txn: *mut MDBX_txn = ptr::null_mut();
    let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn, ptr::null_mut());
    
    if rc != MDBX_SUCCESS {
        return Err(TxnError::BeginFailed(rc));
    }
    
    match operation(txn) {
        Ok(result) => {
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            if rc != MDBX_SUCCESS {
                return Err(TxnError::CommitFailed(rc));
            }
            Ok(result)
        }
        Err(e) => {
            mdbx_txn_abort(txn);
            Err(e)
        }
    }
}

/// Opens a named database (DBI) within a transaction.
///
/// # Parameters
///
/// - `txn`: Active transaction
/// - `name`: Database name (use `None` for unnamed/default database)
/// - `create`: Whether to create the database if it doesn't exist
///
/// # Returns
///
/// Returns the `MDBX_dbi` handle on success.
///
/// # Safety
///
/// The transaction must be valid and active.
pub unsafe fn open_dbi(
    txn: *mut MDBX_txn,
    name: Option<&str>,
    create: bool,
) -> Result<MDBX_dbi, TxnError> {
    let name_c = if let Some(n) = name {
        Some(CString::new(n).map_err(|_| TxnError::InvalidName(n.to_string()))?)
    } else {
        None
    };
    
    let name_ptr = name_c.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null());
    let flags = if create { MDBX_CREATE } else { 0 };
    
    let mut dbi: MDBX_dbi = 0;
    let rc = mdbx_dbi_open(txn, name_ptr, flags, &mut dbi);
    
    if rc != MDBX_SUCCESS {
        return Err(TxnError::DbiOpenFailed {
            name: name.unwrap_or("<unnamed>").to_string(),
            code: rc,
        });
    }
    
    Ok(dbi)
}

/// Opens multiple named databases in a single transaction.
///
/// # Parameters
///
/// - `env`: MDBX environment
/// - `names`: Slice of database names to open
/// - `create`: Whether to create databases if they don't exist
///
/// # Returns
///
/// Returns `Vec<MDBX_dbi>` in the same order as `names`.
///
/// # Safety
///
/// The environment must be valid.
pub unsafe fn open_multiple_dbis(
    env: *mut MDBX_env,
    names: &[&str],
    create: bool,
) -> Result<Vec<MDBX_dbi>, TxnError> {
    with_write_txn(env, |txn| {
        names
            .iter()
            .map(|name| open_dbi(txn, Some(name), create))
            .collect()
    })
}

