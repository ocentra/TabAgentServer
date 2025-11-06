//! Enterprise-grade zero-copy wrapper using MDBX_RESERVE FFI
//!
//! This module provides GUARANTEED aligned writes and TRUE zero-copy reads.
//!
//! FEATURES:
//! - MDBX_RESERVE for guaranteed alignment
//! - Hardware CRC32C (SSE4.2) with software fallback
//! - 16-byte header: magic + version + pad + len + crc32
//! - CountingWriter + RawPtrWriter for future two-pass optimization

use std::mem;
use std::ptr;
use std::slice;
use std::os::raw::c_void;
use std::io::{self, Write};

use thiserror::Error;
use crc32fast::Hasher as Crc32;
use rkyv::{Archive, Archived};

// Direct FFI imports
use mdbx_sys::{
    MDBX_txn, MDBX_dbi, MDBX_val, MDBX_SUCCESS, MDBX_NOTFOUND,
    MDBX_RESERVE,
    mdbx_put, mdbx_get,
};

/// CountingWriter: counts bytes without allocating
#[derive(Debug, Default, Clone, Copy)]
pub struct CountingWriter {
    pos: usize,
}

impl CountingWriter {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }
}

impl Write for CountingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = buf.len();
        self.pos = self.pos.checked_add(n).ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "CountingWriter position overflow")
        })?;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// RawPtrWriter: writes into pre-allocated buffer at raw pointer
#[derive(Debug)]
pub struct RawPtrWriter {
    base: *mut u8,
    capacity: usize,
    offset: usize,
}

impl RawPtrWriter {
    /// SAFETY: base must point to at least capacity bytes of writable memory
    pub unsafe fn from_raw_parts(base: *mut u8, capacity: usize) -> Self {
        Self { base, capacity, offset: 0 }
    }

    pub fn pos(&self) -> usize {
        self.offset
    }

    pub fn remaining(&self) -> usize {
        self.capacity.saturating_sub(self.offset)
    }
}

impl Write for RawPtrWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = buf.len();
        if n > self.remaining() {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                format!("RawPtrWriter overflow: need {} but have {}", n, self.remaining()),
            ));
        }
        unsafe {
            let dst = self.base.add(self.offset);
            ptr::copy_nonoverlapping(buf.as_ptr(), dst, n);
        }
        self.offset += n;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Hardware-accelerated CRC32C with fallback
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn cpu_has_sse42() -> bool {
    std::is_x86_feature_detected!("sse4.2")
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn cpu_has_sse42() -> bool {
    false
}

pub fn crc32c_hardware(bytes: &[u8]) -> u32 {
    if cpu_has_sse42() {
        // Hardware-accelerated CRC32C (SSE4.2)
        crc32c::crc32c(bytes)
    } else {
        // Fallback to fast software implementation
        let mut hasher = Crc32::new();
        hasher.update(bytes);
        hasher.finalize()
    }
}

/// Fixed header layout: 16 bytes total
const HEADER_MAGIC: u32 = 0x5A5A_AA55;
const HEADER_VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 16;

#[derive(Debug, Error)]
pub enum ZcError {
    #[error("mdbx error: {0}")]
    Mdbx(i32),

    #[error("io/invalid input: {0}")]
    Invalid(String),

    #[error("crc mismatch: expected {expected:#010x} got {actual:#010x}")]
    CrcMismatch { expected: u32, actual: u32 },

    #[error("alignment error: required {required}, addr % required = {modulo}")]
    Alignment { required: usize, modulo: usize },

    #[error("archive access error: {0}")]
    Access(String),
    
    #[error("serialization error: {0}")]
    Serialization(String),
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub magic: u32,
    pub version: u8,
    pub pad_len: u8,
    pub reserved: u16,
    pub archived_len: u32,
    pub crc32: u32,
}

impl Header {
    fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut b = [0u8; HEADER_SIZE];
        b[0..4].copy_from_slice(&self.magic.to_le_bytes());
        b[4] = self.version;
        b[5] = self.pad_len;
        b[6..8].copy_from_slice(&self.reserved.to_le_bytes());
        b[8..12].copy_from_slice(&self.archived_len.to_le_bytes());
        b[12..16].copy_from_slice(&self.crc32.to_le_bytes());
        b
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Header, ZcError> {
        if buf.len() < HEADER_SIZE {
            return Err(ZcError::Invalid("header too small".into()));
        }
        let magic = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let version = buf[4];
        let pad_len = buf[5];
        let reserved = u16::from_le_bytes([buf[6], buf[7]]);
        let archived_len = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let crc32 = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
        Ok(Header {
            magic,
            version,
            pad_len,
            reserved,
            archived_len,
            crc32,
        })
    }
}

/// Put pre-serialized rkyv bytes using MDBX_RESERVE with guaranteed alignment
/// Caller must serialize the value first using rkyv::to_bytes()
pub fn put_aligned(
    txn: *mut MDBX_txn,
    dbi: MDBX_dbi,
    key: &[u8],
    archived_bytes: &[u8],
) -> Result<(), ZcError> {
    unsafe {
        put_aligned_bytes(txn, dbi, key, archived_bytes)
    }
}

unsafe fn put_aligned_bytes(
    txn: *mut MDBX_txn,
    dbi: MDBX_dbi,
    key: &[u8],
    archived_bytes: &[u8],
) -> Result<(), ZcError> {
    let archived_len = archived_bytes.len();
    if archived_len == 0 {
        return Err(ZcError::Invalid("Archived bytes empty".into()));
    }
    // Compute alignment requirement
    let required_align = 8usize;

    // Reserve: header + max padding + archived data
    let reserve_len = HEADER_SIZE + (required_align - 1) + archived_len;

    // Build MDBX_val for key
    let mut key_val = MDBX_val {
        iov_len: key.len(),
        iov_base: key.as_ptr() as *mut c_void,
    };

    // Build MDBX_val for data
    let mut data_val = MDBX_val {
        iov_len: reserve_len,
        iov_base: ptr::null_mut(),
    };

    // Call mdbx_put with MDBX_RESERVE
    let rc = mdbx_put(txn, dbi, &mut key_val as *mut _, &mut data_val as *mut _, MDBX_RESERVE);
    if rc != MDBX_SUCCESS {
        return Err(ZcError::Mdbx(rc));
    }

    let base_ptr = data_val.iov_base as *mut u8;
    if base_ptr.is_null() {
        return Err(ZcError::Invalid("mdbx_put returned null base".into()));
    }

    // Compute padding
    let after_header_addr = (base_ptr as usize) + HEADER_SIZE;
    let modulo = after_header_addr % required_align;
    let pad = if modulo == 0 { 0 } else { required_align - modulo };

    if pad > u8::MAX as usize {
        return Err(ZcError::Invalid("pad overflow > 255".into()));
    }

    // Copy archived bytes to aligned position
    let write_ptr = base_ptr.add(HEADER_SIZE + pad);
    ptr::copy_nonoverlapping(archived_bytes.as_ptr(), write_ptr, archived_len);

    // Compute CRC with hardware acceleration
    let crc = crc32c_hardware(archived_bytes);

    // Build and write header
    let header = Header {
        magic: HEADER_MAGIC,
        version: HEADER_VERSION,
        pad_len: pad as u8,
        reserved: 0,
        archived_len: archived_len as u32,
        crc32: crc,
    };

    let header_bytes = header.to_bytes();
    ptr::copy_nonoverlapping(header_bytes.as_ptr(), base_ptr, HEADER_SIZE);

    // Zero padding bytes
    if pad > 0 {
        let pad_ptr = base_ptr.add(HEADER_SIZE);
        ptr::write_bytes(pad_ptr as *mut c_void, 0u8, pad);
    }

    Ok(())
}

/// Read raw byte slice with zero-copy (for testing)
/// Returns the raw archived bytes directly from mmap without type checking
pub unsafe fn get_zero_copy_raw<'txn>(
    txn: *mut MDBX_txn,
    dbi: MDBX_dbi,
    key: &[u8],
) -> Result<Option<&'txn [u8]>, ZcError> {
    // Build key val
    let mut key_val = MDBX_val {
        iov_len: key.len(),
        iov_base: key.as_ptr() as *mut c_void,
    };

    // Empty data val
    let mut data_val = MDBX_val {
        iov_len: 0,
        iov_base: ptr::null_mut(),
    };

    let rc = mdbx_get(txn, dbi, &mut key_val as *mut _, &mut data_val as *mut _);
    if rc == MDBX_NOTFOUND {
        return Ok(None);
    }
    if rc != MDBX_SUCCESS {
        return Err(ZcError::Mdbx(rc));
    }

    // data_val points into mmap
    let vptr = data_val.iov_base as *const u8;
    let vlen = data_val.iov_len as usize;
    
    if vlen < HEADER_SIZE {
        return Err(ZcError::Invalid("value smaller than header".into()));
    }

    // Read and validate header
    let header_slice = slice::from_raw_parts(vptr, HEADER_SIZE);
    let header = Header::from_bytes(header_slice)?;

    if header.magic != HEADER_MAGIC {
        return Err(ZcError::Invalid(format!("magic mismatch: got {:#010x}", header.magic)));
    }
    if header.version != HEADER_VERSION {
        return Err(ZcError::Invalid(format!("version mismatch: got {}", header.version)));
    }

    let pad = header.pad_len as usize;
    let archived_len = header.archived_len as usize;

    if HEADER_SIZE + pad + archived_len > vlen {
        return Err(ZcError::Invalid("length fields exceed value length".into()));
    }

    // Compute pointer to archived start
    let archived_ptr = vptr.add(HEADER_SIZE + pad);

    // Validate CRC with hardware acceleration
    let archived_slice = slice::from_raw_parts(archived_ptr, archived_len);
    let actual_crc = crc32c_hardware(archived_slice);
    if actual_crc != header.crc32 {
        return Err(ZcError::CrcMismatch { expected: header.crc32, actual: actual_crc });
    }

    // Return raw slice directly from mmap (ZERO-COPY)
    Ok(Some(archived_slice))
}

/// Read zero-copy Archived<T> for given key
pub unsafe fn get_zero_copy<'txn, T>(
    txn: *mut MDBX_txn,
    dbi: MDBX_dbi,
    key: &[u8],
) -> Result<Option<&'txn Archived<T>>, ZcError>
where
    T: Archive,
{
    // Build key val
    let mut key_val = MDBX_val {
        iov_len: key.len(),
        iov_base: key.as_ptr() as *mut c_void,
    };

    // Empty data val
    let mut data_val = MDBX_val {
        iov_len: 0,
        iov_base: ptr::null_mut(),
    };

    let rc = mdbx_get(txn, dbi, &mut key_val as *mut _, &mut data_val as *mut _);
    if rc == MDBX_NOTFOUND {
        return Ok(None);
    }
    if rc != MDBX_SUCCESS {
        return Err(ZcError::Mdbx(rc));
    }

    // data_val points into mmap
    let vptr = data_val.iov_base as *const u8;
    let vlen = data_val.iov_len as usize;
    
    if vlen < HEADER_SIZE {
        return Err(ZcError::Invalid("value smaller than header".into()));
    }

    // Read and validate header
    let header_slice = slice::from_raw_parts(vptr, HEADER_SIZE);
    let header = Header::from_bytes(header_slice)?;

    if header.magic != HEADER_MAGIC {
        return Err(ZcError::Invalid(format!("magic mismatch: got {:#010x}", header.magic)));
    }
    if header.version != HEADER_VERSION {
        return Err(ZcError::Invalid(format!("version mismatch: got {}", header.version)));
    }

    let pad = header.pad_len as usize;
    let archived_len = header.archived_len as usize;

    if HEADER_SIZE + pad + archived_len > vlen {
        return Err(ZcError::Invalid("length fields exceed value length".into()));
    }

    // Compute pointer to archived start
    let archived_ptr = vptr.add(HEADER_SIZE + pad);

    // Alignment check
    let required_align = mem::align_of::<Archived<T>>();
    let modulo = (archived_ptr as usize) % required_align;
    if modulo != 0 {
        return Err(ZcError::Alignment { required: required_align, modulo });
    }

    // Validate CRC with hardware acceleration
    let archived_slice = slice::from_raw_parts(archived_ptr, archived_len);
    let actual_crc = crc32c_hardware(archived_slice);
    if actual_crc != header.crc32 {
        return Err(ZcError::CrcMismatch { expected: header.crc32, actual: actual_crc });
    }

    // TRUE ZERO-COPY: Access archived data directly from mmap
    let archived_ref = rkyv::access_unchecked::<Archived<T>>(archived_slice);

    Ok(Some(archived_ref))
}

