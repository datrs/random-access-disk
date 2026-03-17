#![forbid(missing_docs)]
#![cfg_attr(test, deny(warnings))]
#![doc(test(attr(deny(warnings))))]

//! # Continuously read and write to disk, using random offsets and lengths
//! [RandomAccessDisk] is a complete implementation of [random-access-storage](https://docs.rs/random-access-storage)
//! for in-memory storage.
//!
//! See also [random-access-memory](https://docs.rs/random-access-memory) for in-memory storage
//! that can be swapped with this.
//!
//! ## Features
//!
//! ### `sparse` (default)
//!
//! Deleting may create sparse files, on by default. Creation of sparse files is tested on OSX, linux and Windows.
//!
//! **NB**: If this is on, `unsafe` code is used to make direct platform-specific calls!
//!
//! ## Examples
//!
//! Reading, writing, deleting and truncating:
//!
//! ```
//! # tokio_test::block_on(async {
//! # example().await;
//! # });
//! # async fn example() {
//! use random_access_disk::RandomAccessDisk;
//! use random_access_storage::RandomAccess;
//!
//! let path = tempfile::Builder::new()
//!     .prefix("basic")
//!     .tempfile()
//!     .unwrap()
//!     .into_temp_path();
//! let storage = RandomAccessDisk::open(path.to_path_buf()).await.unwrap();
//! storage.write(0, b"hello").await.unwrap();
//! storage.write(5, b" world").await.unwrap();
//! assert_eq!(storage.read(0, 11).await.unwrap(), b"hello world");
//! assert_eq!(storage.len(), 11);
//! storage.del(5, 2).await.unwrap();
//! assert_eq!(storage.read(5, 2).await.unwrap(), [0, 0]);
//! assert_eq!(storage.len(), 11);
//! storage.truncate(2).await.unwrap();
//! assert_eq!(storage.len(), 2);
//! storage.truncate(5).await.unwrap();
//! assert_eq!(storage.len(), 5);
//! assert_eq!(storage.read(0, 5).await.unwrap(), [b'h', b'e', 0, 0, 0]);
//! # }
//! ```
//!
//! In order to get benefits from the swappable interface, you will
//! in most cases want to use generic functions for storage manipulation:
//!
//! ```
//! # tokio_test::block_on(async {
//! # example().await;
//! # });
//! # async fn example() {
//! use random_access_storage::RandomAccess;
//! use random_access_disk::RandomAccessDisk;
//! use std::fmt::Debug;
//!
//! let path = tempfile::Builder::new().prefix("swappable").tempfile().unwrap().into_temp_path();
//! let storage = RandomAccessDisk::open(path.to_path_buf()).await.unwrap();
//! write_hello_world(&storage).await;
//! assert_eq!(read_hello_world(&storage).await, b"hello world");
//!
//! /// Write with swappable storage
//! async fn write_hello_world<T>(storage: &T)
//! where T: RandomAccess + Debug + Send,
//! {
//!   storage.write(0, b"hello").await.unwrap();
//!   storage.write(5, b" world").await.unwrap();
//! }
//!
//! /// Read with swappable storage
//! async fn read_hello_world<T>(storage: &T) -> Vec<u8>
//! where T: RandomAccess + Debug + Send,
//! {
//!   storage.read(0, 11).await.unwrap()
//! }
//! # }

use async_lock::Mutex;
use random_access_storage::{BoxFuture, RandomAccess, RandomAccessError};
use std::{
    io::SeekFrom,
    path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

#[cfg(all(
    feature = "sparse",
    any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "macos",
    )
))]
mod unix;
#[cfg(all(
    feature = "sparse",
    any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "macos",
    )
))]
use unix::{get_length_and_block_size, set_sparse, trim};

#[cfg(all(feature = "sparse", windows))]
mod windows;
#[cfg(all(feature = "sparse", windows))]
use windows::{get_length_and_block_size, set_sparse, trim};

#[cfg(not(all(
    feature = "sparse",
    any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "macos",
        windows,
    )
)))]
mod default;

#[cfg(not(all(
    feature = "sparse",
    any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "macos",
        windows,
    )
)))]
use default::{get_length_and_block_size, set_sparse, trim};

/// Internal mutable state of [RandomAccessDisk].
#[derive(Debug)]
struct DiskInner {
    file: Option<fs::File>,
    length: u64,
    block_size: u64,
    auto_sync: bool,
}

impl DiskInner {
    async fn do_truncate(&mut self, length: u64) -> Result<(), RandomAccessError> {
        self.length = length;
        let auto_sync = self.auto_sync;
        let file = self.file.as_ref().expect("self.file was None.");
        file.set_len(length).await?;
        if auto_sync {
            file.sync_all().await?;
        }
        Ok(())
    }
}

/// Main constructor.
#[derive(Debug, Clone)]
pub struct RandomAccessDisk {
    #[allow(dead_code)]
    filename: path::PathBuf,
    inner: Arc<Mutex<DiskInner>>,
    /// Cached length for synchronous reads via [`RandomAccess::len`].
    length: Arc<AtomicU64>,
}

impl RandomAccessDisk {
    /// Create a new (auto-sync) instance to storage at `filename`.
    #[allow(clippy::new_ret_no_self)]
    pub async fn open(
        filename: impl AsRef<path::Path>,
    ) -> Result<RandomAccessDisk, RandomAccessError> {
        Self::builder(filename).build().await
    }

    /// Initialize a builder with storage at `filename`.
    pub fn builder(filename: impl AsRef<path::Path>) -> Builder {
        Builder::new(filename)
    }
}

impl RandomAccess for RandomAccessDisk {
    fn write(&self, offset: u64, data: &[u8]) -> BoxFuture<Result<(), RandomAccessError>> {
        let inner = self.inner.clone();
        let length_arc = Arc::clone(&self.length);
        let data = data.to_vec();
        Box::pin(async move {
            let mut inner = inner.lock().await;
            let auto_sync = inner.auto_sync;
            let new_len = offset + (data.len() as u64);
            {
                let file = inner.file.as_mut().expect("self.file was None.");
                file.seek(SeekFrom::Start(offset)).await?;
                file.write_all(&data).await?;
                if auto_sync {
                    file.sync_all().await?;
                }
            }
            if new_len > inner.length {
                inner.length = new_len;
                length_arc.store(new_len, Ordering::Relaxed);
            }
            Ok(())
        })
    }

    // NOTE(yw): disabling clippy here because we files on disk might be sparse,
    // and sometimes you might want to read a bit of memory to check if it's
    // formatted or not. Returning zero'd out memory seems like an OK thing to do.
    // We should probably come back to this at a future point, and determine
    // whether it's okay to return a fully zero'd out slice. It's a bit weird,
    // because we're replacing empty data with actual zeroes - which does not
    // reflect the state of the world.
    // #[cfg_attr(test, allow(unused_io_amount))]
    fn read(&self, offset: u64, length: u64) -> BoxFuture<Result<Vec<u8>, RandomAccessError>> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let mut guard = inner.lock().await;
            let stored_length = guard.length;
            if offset + length > stored_length {
                return Err(RandomAccessError::OutOfBounds {
                    offset,
                    end: Some(offset + length),
                    length: stored_length,
                });
            }
            let file = guard.file.as_mut().expect("self.file was None.");
            let mut buffer = vec![0; length as usize];
            file.seek(SeekFrom::Start(offset)).await?;
            let _bytes_read = file.read(&mut buffer[..]).await?;
            Ok(buffer)
        })
    }

    fn del(&self, offset: u64, length: u64) -> BoxFuture<Result<(), RandomAccessError>> {
        let inner = self.inner.clone();
        let length_arc = Arc::clone(&self.length);
        Box::pin(async move {
            let mut inner = inner.lock().await;
            if offset > inner.length {
                return Err(RandomAccessError::OutOfBounds {
                    offset,
                    end: None,
                    length: inner.length,
                });
            };

            if length == 0 {
                // No-op
                return Ok(());
            }

            // Delete is truncate if up to the current length or more is deleted
            if offset + length >= inner.length {
                inner.do_truncate(offset).await?;
                length_arc.store(offset, Ordering::Relaxed);
                return Ok(());
            }

            let auto_sync = inner.auto_sync;
            let block_size = inner.block_size;
            let file = inner.file.as_mut().expect("self.file was None.");
            trim(file, offset, length, block_size).await?;
            if auto_sync {
                file.sync_all().await?;
            }
            Ok(())
        })
    }

    fn truncate(&self, length: u64) -> BoxFuture<Result<(), RandomAccessError>> {
        let inner = self.inner.clone();
        let length_arc = Arc::clone(&self.length);
        Box::pin(async move {
            let mut inner = inner.lock().await;
            inner.do_truncate(length).await?;
            length_arc.store(length, Ordering::Relaxed);
            Ok(())
        })
    }

    fn len(&self) -> u64 {
        self.length.load(Ordering::Relaxed)
    }

    fn sync_all(&self) -> BoxFuture<Result<(), RandomAccessError>> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let inner = inner.lock().await;
            if !inner.auto_sync {
                let file = inner.file.as_ref().expect("self.file was None.");
                file.sync_all().await?;
            }
            Ok(())
        })
    }
}

/// Builder for [RandomAccessDisk]
pub struct Builder {
    filename: path::PathBuf,
    auto_sync: bool,
}

impl Builder {
    /// Create new builder at `path` (with auto-sync true by default).
    pub fn new(filename: impl AsRef<path::Path>) -> Self {
        Self {
            filename: filename.as_ref().into(),
            auto_sync: true,
        }
    }

    /// Set auto-sync.
    // NB: tokio cannot flush on drop (no AsyncDrop yet), so disabling auto_sync
    // means you must call sync_all() explicitly before the value is dropped.
    pub fn auto_sync(mut self, auto_sync: bool) -> Self {
        self.auto_sync = auto_sync;
        self
    }

    /// Build a [RandomAccessDisk] instance
    pub async fn build(self) -> Result<RandomAccessDisk, RandomAccessError> {
        if let Some(dirname) = self.filename.parent() {
            mkdirp::mkdirp(dirname)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&self.filename)
            .await?;
        file.sync_all().await?;

        set_sparse(&mut file).await?;

        let (length, block_size) = get_length_and_block_size(&file).await?;
        let length_arc = Arc::new(AtomicU64::new(length));
        Ok(RandomAccessDisk {
            filename: self.filename,
            inner: Arc::new(Mutex::new(DiskInner {
                file: Some(file),
                length,
                auto_sync: self.auto_sync,
                block_size,
            })),
            length: length_arc,
        })
    }
}
