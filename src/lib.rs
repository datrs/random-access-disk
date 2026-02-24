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
//! ### `async-std` (default)
//!
//! Use the async-std runtime, on by default. Either this or `tokio` is mandatory.
//!
//! ### `tokio`
//!
//! Use the tokio runtime. Either this or `async_std` is mandatory.
//!
//! ## Examples
//!
//! Reading, writing, deleting and truncating:
//!
//! ```
//! # #[cfg(feature = "tokio")]
//! # tokio_test::block_on(async {
//! # example().await;
//! # });
//! # #[cfg(feature = "async-std")]
//! # async_std::task::block_on(async {
//! # example().await;
//! # });
//! # async fn example() {
//! use random_access_storage::RandomAccess;
//! use random_access_disk::RandomAccessDisk;
//!
//! let path = tempfile::Builder::new().prefix("basic").tempfile().unwrap().into_temp_path();
//! let mut storage = RandomAccessDisk::open(path.to_path_buf()).await.unwrap();
//! storage.write(0, b"hello").await.unwrap();
//! storage.write(5, b" world").await.unwrap();
//! assert_eq!(storage.read(0, 11).await.unwrap(), b"hello world");
//! assert_eq!(storage.len().await.unwrap(), 11);
//! storage.del(5, 2).await.unwrap();
//! assert_eq!(storage.read(5, 2).await.unwrap(), [0, 0]);
//! assert_eq!(storage.len().await.unwrap(), 11);
//! storage.truncate(2).await.unwrap();
//! assert_eq!(storage.len().await.unwrap(), 2);
//! storage.truncate(5).await.unwrap();
//! assert_eq!(storage.len().await.unwrap(), 5);
//! assert_eq!(storage.read(0, 5).await.unwrap(), [b'h', b'e', 0, 0, 0]);
//! # }
//! ```
//!
//! In order to get benefits from the swappable interface, you will
//! in most cases want to use generic functions for storage manipulation:
//!
//! ```
//! # #[cfg(feature = "tokio")]
//! # tokio_test::block_on(async {
//! # example().await;
//! # });
//! # #[cfg(feature = "async-std")]
//! # async_std::task::block_on(async {
//! # example().await;
//! # });
//! # async fn example() {
//! use random_access_storage::RandomAccess;
//! use random_access_disk::RandomAccessDisk;
//! use std::fmt::Debug;
//!
//! let path = tempfile::Builder::new().prefix("swappable").tempfile().unwrap().into_temp_path();
//! let mut storage = RandomAccessDisk::open(path.to_path_buf()).await.unwrap();
//! write_hello_world(&mut storage).await;
//! assert_eq!(read_hello_world(&storage).await, b"hello world");
//!
//! /// Write with swappable storage
//! async fn write_hello_world<T>(storage: &mut T)
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

#[cfg(not(any(feature = "async-std", feature = "tokio")))]
compile_error!(
  "Either feature `random-access-disk/async-std` or `random-access-disk/tokio` must be enabled."
);

#[cfg(all(feature = "async-std", feature = "tokio"))]
compile_error!("features `random-access-disk/async-std` and `random-access-disk/tokio` are mutually exclusive");

#[cfg(feature = "async-std")]
use async_std::{
  fs::{self, OpenOptions},
  io::prelude::{SeekExt, WriteExt},
  io::{ReadExt, SeekFrom},
};
use async_lock::Mutex;
use random_access_storage::{BoxFuture, RandomAccess, RandomAccessError};
use std::{ops::Drop, path, sync::Arc};

#[cfg(feature = "tokio")]
use std::io::SeekFrom;
#[cfg(feature = "tokio")]
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

#[async_trait::async_trait]
impl RandomAccess for RandomAccessDisk {
  async fn write(
    &mut self,
    offset: u64,
    data: &[u8],
  ) -> Result<(), RandomAccessError> {
    let mut inner = self.inner.lock().await;
    let auto_sync = inner.auto_sync;
    let new_len = offset + (data.len() as u64);
    {
      let file = inner.file.as_mut().expect("self.file was None.");
      file.seek(SeekFrom::Start(offset)).await?;
      file.write_all(data).await?;
      if auto_sync {
        file.sync_all().await?;
      }
    }
    if new_len > inner.length {
      inner.length = new_len;
    }
    Ok(())
  }

  // NOTE(yw): disabling clippy here because we files on disk might be sparse,
  // and sometimes you might want to read a bit of memory to check if it's
  // formatted or not. Returning zero'd out memory seems like an OK thing to do.
  // We should probably come back to this at a future point, and determine
  // whether it's okay to return a fully zero'd out slice. It's a bit weird,
  // because we're replacing empty data with actual zeroes - which does not
  // reflect the state of the world.
  // #[cfg_attr(test, allow(unused_io_amount))]
  fn read(
    &self,
    offset: u64,
    length: u64,
  ) -> BoxFuture<Result<Vec<u8>, RandomAccessError>> {
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

  async fn del(
    &mut self,
    offset: u64,
    length: u64,
  ) -> Result<(), RandomAccessError> {
    let mut inner = self.inner.lock().await;
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
      return inner.do_truncate(offset).await;
    }

    let auto_sync = inner.auto_sync;
    let block_size = inner.block_size;
    let file = inner.file.as_mut().expect("self.file was None.");
    trim(file, offset, length, block_size).await?;
    if auto_sync {
      file.sync_all().await?;
    }
    Ok(())
  }

  async fn truncate(&mut self, length: u64) -> Result<(), RandomAccessError> {
    self.inner.lock().await.do_truncate(length).await
  }

  async fn len(&mut self) -> Result<u64, RandomAccessError> {
    Ok(self.inner.lock().await.length)
  }

  async fn is_empty(&mut self) -> Result<bool, RandomAccessError> {
    Ok(self.inner.lock().await.length == 0)
  }

  async fn sync_all(&mut self) -> Result<(), RandomAccessError> {
    let inner = self.inner.lock().await;
    if !inner.auto_sync {
      let file = inner.file.as_ref().expect("self.file was None.");
      file.sync_all().await?;
    }
    Ok(())
  }
}

impl Drop for RandomAccessDisk {
  fn drop(&mut self) {
    // We need to flush the file on drop. We attempt a non-blocking try_lock
    // so we don't deadlock if a lock is somehow still held; in practice this
    // succeeds since Drop runs after all borrows are released.
    #[cfg(feature = "async-std")]
    if let Some(guard) = self.inner.try_lock() {
      if let Some(file) = &guard.file {
        let _ = async_std::task::block_on(file.sync_all());
      }
    }
    // For tokio, block_on inside a running runtime panics, so we skip it.
    // See the original comment for details; this will be resolved by AsyncDrop.
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

  /// Set auto-sync
  // NB: Because of no AsyncDrop, tokio can not ensure that changes are synced when dropped,
  // see impl Drop above.
  #[cfg(feature = "async-std")]
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
      .read(true)
      .write(true)
      .open(&self.filename)
      .await?;
    file.sync_all().await?;

    set_sparse(&mut file).await?;

    let (length, block_size) = get_length_and_block_size(&file).await?;
    Ok(RandomAccessDisk {
      filename: self.filename,
      inner: Arc::new(Mutex::new(DiskInner {
        file: Some(file),
        length,
        auto_sync: self.auto_sync,
        block_size,
      })),
    })
  }
}
