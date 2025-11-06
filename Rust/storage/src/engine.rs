use common::DbResult;

pub trait ReadGuard: Send {
    fn data(&self) -> &[u8];
    
    fn archived<T: rkyv::Archive>(&self) -> Result<&T::Archived, String>
    where
        <T as rkyv::Archive>::Archived: 'static;
}

pub trait StorageEngine: Send + Sync + Clone + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    type Transaction: StorageTransaction<Error = Self::Error>;
    type ReadGuard: ReadGuard;
    
    fn open(path: &str) -> DbResult<Self>
    where
        Self: Sized;
    
    fn get<'a>(&'a self, tree: &str, key: &[u8]) -> Result<Option<Self::ReadGuard>, Self::Error>;
    fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error>;
    fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn open_tree(&self, name: &str) -> Result<(), Self::Error>;
    fn scan_prefix<'a>(&'a self, tree: &str, prefix: &[u8]) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + 'a>;
    fn iter<'a>(&'a self, tree: &str) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + 'a>;
    fn transaction(&self) -> Result<Self::Transaction, Self::Error>;
    fn flush(&self) -> Result<(), Self::Error>;
    fn size_on_disk(&self) -> Result<u64, Self::Error>;
}

pub trait StorageTransaction: Send {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error>;
    fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn commit(self) -> Result<(), Self::Error>;
    fn abort(self) -> Result<(), Self::Error>;
}

mod mdbx_engine {
    use super::*;
    use mdbx_base::zero_copy_ffi;
    use dashmap::DashMap;
    use std::sync::Arc;
    use std::ptr;
    use std::ffi::CString;
    use std::os::raw::c_void;
    use mdbx_base::mdbx_sys::{
        MDBX_env, MDBX_txn, MDBX_dbi, MDBX_val, MDBX_SUCCESS, MDBX_NOTFOUND,
        MDBX_TXN_RDONLY, MDBX_CREATE, MDBX_FIRST, MDBX_NEXT,
        mdbx_env_create, mdbx_env_set_geometry, mdbx_env_open,
        mdbx_txn_begin_ex, mdbx_txn_commit_ex, mdbx_txn_abort,
        mdbx_dbi_open, mdbx_del, mdbx_env_sync_ex, mdbx_env_close_ex,
        mdbx_cursor_open, mdbx_cursor_close, mdbx_cursor_get,
        MDBX_cursor,
        mdbx_env_set_option, MDBX_opt_max_db, mdbx_get,
    };
    
    pub struct MdbxReadGuard {
        txn: *mut MDBX_txn,
        dbi: MDBX_dbi,
        key: Vec<u8>,
        data_ptr: *const u8,
        data_len: usize,
    }
    
    impl Drop for MdbxReadGuard {
        fn drop(&mut self) {
            unsafe {
                if !self.txn.is_null() {
                    mdbx_txn_abort(self.txn);
                }
            }
        }
    }
    
    unsafe impl Send for MdbxReadGuard {}
    unsafe impl Sync for MdbxReadGuard {}
    
    impl ReadGuard for MdbxReadGuard {
        fn data(&self) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts(self.data_ptr, self.data_len)
            }
        }
        
        fn archived<T: rkyv::Archive>(&self) -> Result<&T::Archived, String>
        where
            <T as rkyv::Archive>::Archived: 'static,
        {
            unsafe {
                zero_copy_ffi::get_zero_copy::<T>(self.txn, self.dbi, &self.key)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "Data disappeared during read".to_string())
            }
        }
    }
    
    pub struct MdbxEngine {
        env: *mut MDBX_env,
        path: Arc<String>,
        dbis: Arc<DashMap<String, MDBX_dbi>>,
    }
    
    unsafe impl Send for MdbxEngine {}
    unsafe impl Sync for MdbxEngine {}
    
    impl Clone for MdbxEngine {
        fn clone(&self) -> Self {
            Self {
                env: self.env,
                path: Arc::clone(&self.path),
                dbis: Arc::clone(&self.dbis),
            }
        }
    }
    
    impl Drop for MdbxEngine {
        fn drop(&mut self) {
            if Arc::strong_count(&self.path) == 1 {
                unsafe {
                    if !self.env.is_null() {
                        mdbx_env_close_ex(self.env, false);
                    }
                }
            }
        }
    }
    
    impl MdbxEngine {
        /// Get the database path
        pub fn db_path(&self) -> Option<String> {
            Some((*self.path).clone())
        }
        
        /// UNSAFE: Get raw MDBX environment pointer for indexing service
        ///
        /// # Safety
        ///
        /// The caller MUST ensure that:
        /// - The returned pointer is only used while this MdbxEngine exists
        /// - No mdbx_env_close is called on this pointer
        /// - All operations on this env follow MDBX's thread-safety rules
        ///
        /// This is intended ONLY for indexing services that need to create
        /// additional DBIs in the same MDBX environment.
        pub unsafe fn get_raw_env(&self) -> *mut MDBX_env {
            self.env
        }
        
        /// Get or create a DBI (table) in this database
        ///
        /// If the DBI already exists, returns the cached handle.
        /// Otherwise, creates a new DBI with MDBX_CREATE flag.
        ///
        /// # Arguments
        ///
        /// * `name` - Name of the table/DBI to get or create
        ///
        /// # Errors
        ///
        /// Returns error if DBI creation fails
        pub fn get_or_create_dbi(&self, name: &str) -> DbResult<MDBX_dbi> {
            // Check if already exists in cache
            if let Some(dbi) = self.dbis.get(name) {
                return Ok(*dbi);
            }
            
            // Create new DBI
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(common::DbError::InvalidOperation(format!(
                        "Failed to begin txn for DBI creation: {}", rc
                    )));
                }
                
                let name_c = CString::new(name)
                    .map_err(|e| common::DbError::InvalidOperation(format!("Invalid DBI name: {}", e)))?;
                
                let mut dbi: MDBX_dbi = 0;
                let rc = mdbx_dbi_open(txn, name_c.as_ptr(), MDBX_CREATE, &mut dbi as *mut _);
                if rc != MDBX_SUCCESS {
                    mdbx_txn_abort(txn);
                    return Err(common::DbError::InvalidOperation(format!(
                        "mdbx_dbi_open failed for {}: {}", name, rc
                    )));
                }
                
                let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(common::DbError::InvalidOperation(format!(
                        "Failed to commit DBI creation: {}", rc
                    )));
                }
                
                // Cache the DBI
                self.dbis.insert(name.to_string(), dbi);
                Ok(dbi)
            }
        }
    }
    
    impl StorageEngine for MdbxEngine {
        type Error = MdbxEngineError;
        type Transaction = MdbxTransaction;
        type ReadGuard = MdbxReadGuard;
        
        fn open(path: &str) -> DbResult<Self> {
            unsafe {
                let mut env: *mut MDBX_env = ptr::null_mut();
                
                let rc = mdbx_env_create(&mut env as *mut _);
                if rc != MDBX_SUCCESS {
                    return Err(common::DbError::InvalidOperation(format!("mdbx_env_create failed: {}", rc)));
                }
                
                // Set max_db limit (REQUIRED to avoid -30791 MDBX_DBS_FULL error)
                // We use 5 DBIs: nodes, edges, embeddings, metadata, chunks
                // Setting to 10 for headroom
                let rc = mdbx_env_set_option(env, MDBX_opt_max_db, 10);
                if rc != MDBX_SUCCESS {
                    mdbx_env_close_ex(env, false);
                    return Err(common::DbError::InvalidOperation(format!("mdbx_env_set_option(max_db) failed: {}", rc)));
                }
                
                let mapsize: isize = 10 * 1024 * 1024 * 1024;
                let rc = mdbx_env_set_geometry(env, -1, -1, mapsize, -1, -1, -1);
                if rc != MDBX_SUCCESS {
                    mdbx_env_close_ex(env, false);
                    return Err(common::DbError::InvalidOperation(format!("mdbx_env_set_geometry failed: {}", rc)));
                }
                
                std::fs::create_dir_all(path)
                    .map_err(|e| common::DbError::InvalidOperation(format!("Failed to create directory: {}", e)))?;
                
                let path_c = CString::new(path)
                    .map_err(|e| common::DbError::InvalidOperation(format!("Invalid path: {}", e)))?;
                
                let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
                if rc != MDBX_SUCCESS {
                    mdbx_env_close_ex(env, false);
                    return Err(common::DbError::InvalidOperation(format!("mdbx_env_open failed: {}", rc)));
                }
                
                let dbis = Arc::new(DashMap::new());
                
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    mdbx_env_close_ex(env, false);
                    return Err(common::DbError::InvalidOperation(format!("mdbx_txn_begin_ex failed: {}", rc)));
                }
                
                for table_name in &["nodes", "edges", "embeddings", "metadata", "chunks"] {
                    let name_c = CString::new(*table_name).unwrap();
                    let mut dbi: MDBX_dbi = 0;
                    let rc = mdbx_dbi_open(txn, name_c.as_ptr(), MDBX_CREATE, &mut dbi as *mut _);
                    if rc != MDBX_SUCCESS {
                        mdbx_txn_abort(txn);
                        mdbx_env_close_ex(env, false);
                        return Err(common::DbError::InvalidOperation(format!("mdbx_dbi_open failed for {}: {}", table_name, rc)));
                    }
                    dbis.insert(table_name.to_string(), dbi);
                }
                
                let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    mdbx_env_close_ex(env, false);
                    return Err(common::DbError::InvalidOperation(format!("mdbx_txn_commit_ex failed: {}", rc)));
                }
                
                Ok(Self {
                    env,
                    path: Arc::new(path.to_string()),
                    dbis,
                })
            }
        }
        
        fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Self::ReadGuard>, Self::Error> {
            let dbi = self.dbis.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?
                .value().clone();
            
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), MDBX_TXN_RDONLY, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_begin failed: {}", rc)));
                }
                
                let mut key_val = MDBX_val {
                    iov_len: key.len(),
                    iov_base: key.as_ptr() as *mut c_void,
                };
                
                let mut data_val = MDBX_val {
                    iov_len: 0,
                    iov_base: ptr::null_mut(),
                };
                
                let rc = mdbx_get(txn, dbi, &mut key_val as *mut _, &mut data_val as *mut _);
                
                if rc == MDBX_NOTFOUND {
                    mdbx_txn_abort(txn);
                    return Ok(None);
                }
                
                if rc != MDBX_SUCCESS {
                    mdbx_txn_abort(txn);
                    return Err(MdbxEngineError::MdbxError(format!("mdbx_get failed: {}", rc)));
                }
                
                let vptr = data_val.iov_base as *const u8;
                let vlen = data_val.iov_len;
                
                if vlen < zero_copy_ffi::HEADER_SIZE {
                    mdbx_txn_abort(txn);
                    return Err(MdbxEngineError::ZeroCopy(zero_copy_ffi::ZcError::Invalid("value too small for header".into())));
                }
                
                let header = zero_copy_ffi::Header::from_bytes(std::slice::from_raw_parts(vptr, zero_copy_ffi::HEADER_SIZE))
                    .map_err(|e| {
                        mdbx_txn_abort(txn);
                        MdbxEngineError::ZeroCopy(e)
                    })?;
                
                let archived_len = header.archived_len as usize;
                let pad = header.pad_len as usize;
                let archived_ptr = vptr.add(zero_copy_ffi::HEADER_SIZE + pad);
                let archived_slice = std::slice::from_raw_parts(archived_ptr, archived_len);
                
                let actual_crc = zero_copy_ffi::crc32c_hardware(archived_slice);
                if actual_crc != header.crc32 {
                    mdbx_txn_abort(txn);
                    return Err(MdbxEngineError::ZeroCopy(zero_copy_ffi::ZcError::CrcMismatch {
                        expected: header.crc32,
                        actual: actual_crc,
                    }));
                }
                
                Ok(Some(MdbxReadGuard {
                    txn,
                    dbi,
                    key: key.to_vec(),
                    data_ptr: archived_ptr,
                    data_len: archived_len,
                }))
            }
        }
        
        fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error> {
            let dbi = self.dbis.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?
                .value().clone();
            
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_begin failed: {}", rc)));
                }
                
                match zero_copy_ffi::put_aligned(txn, dbi, key, &value) {
                    Ok(()) => {
                        let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
                        if rc != MDBX_SUCCESS {
                            return Err(MdbxEngineError::MdbxError(format!("txn_commit failed: {}", rc)));
                        }
                        Ok(())
                    }
                    Err(e) => {
                        mdbx_txn_abort(txn);
                        Err(MdbxEngineError::ZeroCopy(e))
                    }
                }
            }
        }
        
        fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            let dbi = self.dbis.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?
                .value().clone();
            
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_begin failed: {}", rc)));
                }
                
                let mut key_val = MDBX_val {
                    iov_len: key.len(),
                    iov_base: key.as_ptr() as *mut c_void,
                };
                
                let mut data_val = MDBX_val {
                    iov_len: 0,
                    iov_base: ptr::null_mut(),
                };
                
                let rc_get = mdbx_get(txn, dbi, &mut key_val as *mut _, &mut data_val as *mut _);
                
                let old_value = if rc_get == MDBX_SUCCESS {
                    let vptr = data_val.iov_base as *const u8;
                    let vlen = data_val.iov_len;
                    Some(std::slice::from_raw_parts(vptr, vlen).to_vec())
                } else {
                    None
                };
                
                let rc = mdbx_del(txn, dbi, &mut key_val as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS && rc != MDBX_NOTFOUND {
                    mdbx_txn_abort(txn);
                    return Err(MdbxEngineError::MdbxError(format!("mdbx_del failed: {}", rc)));
                }
                
                let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_commit failed: {}", rc)));
                }
                
                Ok(old_value)
            }
        }
        
        fn open_tree(&self, name: &str) -> Result<(), Self::Error> {
            if self.dbis.contains_key(name) {
                return Ok(());
            }
            
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_begin failed: {}", rc)));
                }
                
                let name_c = CString::new(name)
                    .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
                
                let mut dbi: MDBX_dbi = 0;
                let rc = mdbx_dbi_open(txn, name_c.as_ptr(), MDBX_CREATE, &mut dbi as *mut _);
                if rc != MDBX_SUCCESS {
                    mdbx_txn_abort(txn);
                    return Err(MdbxEngineError::MdbxError(format!("mdbx_dbi_open failed: {}", rc)));
                }
                
                let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_commit failed: {}", rc)));
                }
                
                self.dbis.insert(name.to_string(), dbi);
                Ok(())
            }
        }
        
        fn scan_prefix<'a>(&'a self, tree: &str, prefix: &[u8]) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + 'a> {
            let dbi = match self.dbis.get(tree) {
                Some(d) => d.value().clone(),
                None => return Box::new(std::iter::once(Err(MdbxEngineError::TreeNotFound(tree.to_string())))),
            };
            
            let prefix_owned = prefix.to_vec();
            let tree_owned = tree.to_string();
            
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), MDBX_TXN_RDONLY, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(format!("txn_begin failed: {}", rc)))));
                }
                
                let mut cursor: *mut MDBX_cursor = ptr::null_mut();
                let rc = mdbx_cursor_open(txn, dbi, &mut cursor as *mut _);
                if rc != MDBX_SUCCESS {
                    mdbx_txn_abort(txn);
                    return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(format!("cursor_open failed: {}", rc)))));
                }
                
                let mut keys = Vec::new();
                let mut key_val = MDBX_val { iov_len: 0, iov_base: ptr::null_mut() };
                let mut data_val = MDBX_val { iov_len: 0, iov_base: ptr::null_mut() };
                
                let rc = mdbx_cursor_get(cursor, &mut key_val as *mut _, &mut data_val as *mut _, MDBX_FIRST);
                if rc == MDBX_SUCCESS {
                    loop {
                        let k_ptr = key_val.iov_base as *const u8;
                        let k_len = key_val.iov_len;
                        let key_slice = std::slice::from_raw_parts(k_ptr, k_len);
                        
                        if key_slice.starts_with(&prefix_owned) {
                            keys.push(key_slice.to_vec());
                        }
                        
                        let rc = mdbx_cursor_get(cursor, &mut key_val as *mut _, &mut data_val as *mut _, MDBX_NEXT);
                        if rc != MDBX_SUCCESS {
                            break;
                        }
                    }
                }
                
                mdbx_cursor_close(cursor);
                mdbx_txn_abort(txn);
                
                Box::new(keys.into_iter().filter_map(move |key| {
                    match self.get(&tree_owned, &key) {
                        Ok(Some(guard)) => Some(Ok((key, guard))),
                        Ok(None) => None,
                        Err(e) => Some(Err(e)),
                    }
                }))
            }
        }
        
        fn iter<'a>(&'a self, tree: &str) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + 'a> {
            self.scan_prefix(tree, &[])
        }
        
        fn transaction(&self) -> Result<Self::Transaction, Self::Error> {
            Ok(MdbxTransaction {
                env: self.env,
                dbis: Arc::clone(&self.dbis),
            })
        }
        
        fn flush(&self) -> Result<(), Self::Error> {
            unsafe {
                let rc = mdbx_env_sync_ex(self.env, true, false);
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("mdbx_env_sync_ex failed: {}", rc)));
                }
                Ok(())
            }
        }
        
        fn size_on_disk(&self) -> Result<u64, Self::Error> {
            Ok(0)
        }
    }
    
    pub struct MdbxTransaction {
        env: *mut MDBX_env,
        dbis: Arc<DashMap<String, MDBX_dbi>>,
    }
    
    unsafe impl Send for MdbxTransaction {}
    unsafe impl Sync for MdbxTransaction {}
    
    impl StorageTransaction for MdbxTransaction {
        type Error = MdbxEngineError;
        
        fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            let dbi = self.dbis.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?
                .value().clone();
            
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), MDBX_TXN_RDONLY, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_begin failed: {}", rc)));
                }
                
                let mut key_val = MDBX_val {
                    iov_len: key.len(),
                    iov_base: key.as_ptr() as *mut c_void,
                };
                
                let mut data_val = MDBX_val {
                    iov_len: 0,
                    iov_base: ptr::null_mut(),
                };
                
                let rc = mdbx_get(txn, dbi, &mut key_val as *mut _, &mut data_val as *mut _);
                
                let result = if rc == MDBX_SUCCESS {
                    let vptr = data_val.iov_base as *const u8;
                    let vlen = data_val.iov_len;
                    Some(std::slice::from_raw_parts(vptr, vlen).to_vec())
                } else {
                    None
                };
                
                mdbx_txn_abort(txn);
                Ok(result)
            }
        }
        
        fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error> {
            let dbi = self.dbis.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?
                .value().clone();
            
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_begin failed: {}", rc)));
                }
                
                match zero_copy_ffi::put_aligned(txn, dbi, key, &value) {
                    Ok(()) => {
                        let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
                        if rc != MDBX_SUCCESS {
                            return Err(MdbxEngineError::MdbxError(format!("txn_commit failed: {}", rc)));
                        }
                        Ok(())
                    }
                    Err(e) => {
                        mdbx_txn_abort(txn);
                        Err(MdbxEngineError::ZeroCopy(e))
                    }
                }
            }
        }
        
        fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            let dbi = self.dbis.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?
                .value().clone();
            
            unsafe {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(self.env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_begin failed: {}", rc)));
                }
                
                let mut key_val = MDBX_val {
                    iov_len: key.len(),
                    iov_base: key.as_ptr() as *mut c_void,
                };
                
                let mut data_val = MDBX_val {
                    iov_len: 0,
                    iov_base: ptr::null_mut(),
                };
                
                let rc_get = mdbx_get(txn, dbi, &mut key_val as *mut _, &mut data_val as *mut _);
                
                let old_value = if rc_get == MDBX_SUCCESS {
                    let vptr = data_val.iov_base as *const u8;
                    let vlen = data_val.iov_len;
                    Some(std::slice::from_raw_parts(vptr, vlen).to_vec())
                } else {
                    None
                };
                
                let rc = mdbx_del(txn, dbi, &mut key_val as *mut _, ptr::null_mut());
                if rc != MDBX_SUCCESS && rc != MDBX_NOTFOUND {
                    mdbx_txn_abort(txn);
                    return Err(MdbxEngineError::MdbxError(format!("mdbx_del failed: {}", rc)));
                }
                
                let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
                if rc != MDBX_SUCCESS {
                    return Err(MdbxEngineError::MdbxError(format!("txn_commit failed: {}", rc)));
                }
                
                Ok(old_value)
            }
        }
        
        fn commit(self) -> Result<(), Self::Error> {
            Ok(())
        }
        
        fn abort(self) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    
    #[derive(Debug)]
    pub enum MdbxEngineError {
        TreeNotFound(String),
        MdbxError(String),
        ZeroCopy(zero_copy_ffi::ZcError),
    }
    
    impl std::fmt::Display for MdbxEngineError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                MdbxEngineError::TreeNotFound(name) => write!(f, "Tree not found: {}", name),
                MdbxEngineError::MdbxError(msg) => write!(f, "mdbx error: {}", msg),
                MdbxEngineError::ZeroCopy(e) => write!(f, "Zero-copy error: {}", e),
            }
        }
    }
    
    impl std::error::Error for MdbxEngineError {}
    
    impl From<MdbxEngineError> for common::DbError {
        fn from(err: MdbxEngineError) -> Self {
            common::DbError::InvalidOperation(err.to_string())
        }
    }
}

pub use mdbx_engine::{MdbxEngine, MdbxTransaction, MdbxEngineError};
