//! Byte backends for profile / File trait payloads.
//!
//! Valence stores metadata and an opaque [`storage_path`](super) key; this trait
//! puts and gets the bytes. Embedded hosts use [`LocalDiskBlobStore`]; fleet can
//! later plug an S3 implementation without changing `ProfilePhoto`.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Errors from [`FileByteBackend`] operations.
#[derive(Debug, Error)]
pub enum FileStoreError {
    /// Object key was missing or not readable.
    #[error("not found")]
    NotFound,
    /// Filesystem or I/O failure (details stay internal).
    #[error("storage I/O error")]
    Io,
    /// Key escapes the store root or is otherwise invalid.
    #[error("invalid storage key")]
    InvalidKey,
}

/// Put / get / delete opaque object keys for File-backed records.
#[async_trait]
pub trait FileByteBackend: Send + Sync {
    /// Store `bytes` under `key` (overwrite if present).
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), FileStoreError>;
    /// Load bytes for `key`.
    async fn get(&self, key: &str) -> Result<Vec<u8>, FileStoreError>;
    /// Delete `key` if present (missing key is ok).
    async fn delete(&self, key: &str) -> Result<(), FileStoreError>;
}

/// Local directory blob store (`uploads/` by default).
#[derive(Debug, Clone)]
pub struct LocalDiskBlobStore {
    root: PathBuf,
}

impl LocalDiskBlobStore {
    /// Create a store rooted at `root` (created on first put).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Default process-local `uploads` directory.
    pub fn default_uploads() -> Self {
        Self::new("uploads")
    }

    fn resolve(&self, key: &str) -> Result<PathBuf, FileStoreError> {
        // Flat object keys only (e.g. `{uuid}.png`); no path separators.
        if key.is_empty()
            || key.contains("..")
            || key.contains('/')
            || key.contains('\\')
            || Path::new(key).is_absolute()
        {
            return Err(FileStoreError::InvalidKey);
        }
        Ok(self.root.join(key))
    }
}

#[async_trait]
impl FileByteBackend for LocalDiskBlobStore {
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), FileStoreError> {
        let path = self.resolve(key)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|_| FileStoreError::Io)?;
        }
        let mut file = fs::File::create(&path)
            .await
            .map_err(|_| FileStoreError::Io)?;
        file.write_all(bytes)
            .await
            .map_err(|_| FileStoreError::Io)?;
        file.flush().await.map_err(|_| FileStoreError::Io)?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, FileStoreError> {
        let path = self.resolve(key)?;
        fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FileStoreError::NotFound
            } else {
                FileStoreError::Io
            }
        })
    }

    async fn delete(&self, key: &str) -> Result<(), FileStoreError> {
        let path = self.resolve(key)?;
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(FileStoreError::Io),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (PathBuf, LocalDiskBlobStore) {
        let root = std::env::temp_dir().join(format!(
            "lepton-file-store-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        (root.clone(), LocalDiskBlobStore::new(root))
    }

    #[tokio::test]
    async fn local_disk_put_get_delete_happy() {
        let (root, store) = temp_store();
        store.put("a.png", b"hello").await.unwrap();
        assert_eq!(store.get("a.png").await.unwrap(), b"hello");
        store.delete("a.png").await.unwrap();
        assert!(matches!(
            store.get("a.png").await.unwrap_err(),
            FileStoreError::NotFound
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn local_disk_rejects_path_escape_sad() {
        let (root, store) = temp_store();
        assert!(matches!(
            store.put("../x.png", b"x").await.unwrap_err(),
            FileStoreError::InvalidKey
        ));
        let _ = std::fs::remove_dir_all(root);
    }
}
