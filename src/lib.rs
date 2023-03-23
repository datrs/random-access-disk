#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(test, deny(warnings))]

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
use random_access_storage::{RandomAccess, RandomAccessError};
use std::ops::Drop;
use std::path;

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

/// Main constructor.
#[derive(Debug)]
pub struct RandomAccessDisk {
  #[allow(dead_code)]
  filename: path::PathBuf,
  file: Option<fs::File>,
  length: u64,
  block_size: u64,
  auto_sync: bool,
}

impl RandomAccessDisk {
  /// Create a new instance.
  #[allow(clippy::new_ret_no_self)]
  pub async fn open(
    filename: path::PathBuf,
  ) -> Result<RandomAccessDisk, RandomAccessError> {
    Self::builder(filename).build().await
  }

  pub fn builder(filename: path::PathBuf) -> Builder {
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
    let file = self.file.as_mut().expect("self.file was None.");
    file.seek(SeekFrom::Start(offset)).await?;
    file.write_all(&data).await?;
    if self.auto_sync {
      file.sync_all().await?;
    }

    // We've changed the length of our file.
    let new_len = offset + (data.len() as u64);
    if new_len > self.length {
      self.length = new_len;
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
  async fn read(
    &mut self,
    offset: u64,
    length: u64,
  ) -> Result<Vec<u8>, RandomAccessError> {
    if (offset + length) as u64 > self.length {
      return Err(RandomAccessError::OutOfBounds {
        offset,
        end: Some(offset + length),
        length: self.length,
      });
    }

    let file = self.file.as_mut().expect("self.file was None.");
    let mut buffer = vec![0; length as usize];
    file.seek(SeekFrom::Start(offset)).await?;
    let _bytes_read = file.read(&mut buffer[..]).await?;
    Ok(buffer)
  }

  async fn del(
    &mut self,
    offset: u64,
    length: u64,
  ) -> Result<(), RandomAccessError> {
    if offset > self.length {
      return Err(RandomAccessError::OutOfBounds {
        offset,
        end: None,
        length: self.length,
      });
    };

    if length == 0 {
      // No-op
      return Ok(());
    }

    // Delete is truncate if up to the current length or more is deleted
    if offset + length >= self.length {
      return self.truncate(offset).await;
    }

    let mut file = self.file.as_mut().expect("self.file was None.");
    trim(&mut file, offset, length, self.block_size).await?;
    if self.auto_sync {
      file.sync_all().await?;
    }
    Ok(())
  }

  async fn truncate(&mut self, length: u64) -> Result<(), RandomAccessError> {
    let file = self.file.as_ref().expect("self.file was None.");
    self.length = length as u64;
    file.set_len(self.length).await?;
    if self.auto_sync {
      file.sync_all().await?;
    }
    Ok(())
  }

  async fn len(&mut self) -> Result<u64, RandomAccessError> {
    Ok(self.length)
  }

  async fn is_empty(&mut self) -> Result<bool, RandomAccessError> {
    Ok(self.length == 0)
  }

  async fn sync_all(&mut self) -> Result<(), RandomAccessError> {
    if !self.auto_sync {
      let file = self.file.as_ref().expect("self.file was None.");
      file.sync_all().await?;
    }
    Ok(())
  }
}

impl Drop for RandomAccessDisk {
  fn drop(&mut self) {
    // We need to flush the file on drop. Unfortunately, that is not possible to do in a
    // non-blocking fashion, but our only other option here is losing data remaining in the
    // write cache. Good task schedulers should be resilient to occasional blocking hiccups in
    // file destructors so we don't expect this to be a common problem in practice.
    // (from async_std::fs::File::drop)
    #[cfg(feature = "async-std")]
    if let Some(file) = &self.file {
      let _ = async_std::task::block_on(file.sync_all());
    }
    // For tokio, the below errors with:
    //
    // "Cannot start a runtime from within a runtime. This happens because a function (like
    // `block_on`) attempted to block the current thread while the thread is being used to
    // drive asynchronous tasks."
    //
    // There doesn't seem to be an equivalent block_on version for tokio that actually works
    // in a sync drop(), so for tokio, we'll need to wait for a real AsyncDrop to arrive.
    //
    // #[cfg(feature = "tokio")]
    // if let Some(file) = &self.file {
    //   tokio::runtime::Handle::current()
    //     .block_on(file.sync_all())
    //     .expect("Could not sync file changes on drop.");
    // }
  }
}

pub struct Builder {
  filename: path::PathBuf,
  auto_sync: bool,
}

impl Builder {
  pub fn new(filename: path::PathBuf) -> Self {
    Self {
      filename,
      auto_sync: true,
    }
  }

  // NB: Because of no AsyncDrop, tokio can not ensure that changes are synced when dropped,
  // see impl Drop above.
  #[cfg(feature = "async-std")]
  pub fn auto_sync(mut self, auto_sync: bool) -> Self {
    self.auto_sync = auto_sync;
    self
  }

  pub async fn build(self) -> Result<RandomAccessDisk, RandomAccessError> {
    if let Some(dirname) = self.filename.parent() {
      mkdirp::mkdirp(&dirname)?;
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
      file: Some(file),
      length,
      auto_sync: self.auto_sync,
      block_size,
    })
  }
}
