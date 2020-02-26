#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(test, deny(warnings))]

use anyhow::{anyhow, Error};
use async_std::fs::{self, OpenOptions};
use async_std::io::prelude::{SeekExt, WriteExt};
use async_std::io::{ReadExt, SeekFrom};
use random_access_storage::RandomAccess;
use std::ops::Drop;
use std::path;

/// Main constructor.
#[derive(Debug)]
pub struct RandomAccessDisk {
  filename: path::PathBuf,
  file: Option<fs::File>,
  length: u64,
  auto_sync: bool,
}

impl RandomAccessDisk {
  /// Create a new instance.
  #[allow(clippy::new_ret_no_self)]
  pub async fn open(
    filename: path::PathBuf,
  ) -> Result<RandomAccessDisk, Error> {
    Self::builder(filename).build().await
  }

  pub fn builder(filename: path::PathBuf) -> Builder {
    Builder::new(filename)
  }
}

#[async_trait::async_trait]
impl RandomAccess for RandomAccessDisk {
  type Error = Box<dyn std::error::Error + Sync + Send>;

  async fn write(
    &mut self,
    offset: u64,
    data: &[u8],
  ) -> Result<(), Self::Error> {
    let mut file = self.file.as_ref().expect("self.file was None.");
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
  ) -> Result<Vec<u8>, Self::Error> {
    if (offset + length) as u64 > self.length {
      return Err(
        anyhow!(
          "Read bounds exceeded. {} < {}..{}",
          self.length,
          offset,
          offset + length
        )
        .into(),
      );
    }

    let mut file = self.file.as_ref().expect("self.file was None.");
    let mut buffer = vec![0; length as usize];
    file.seek(SeekFrom::Start(offset)).await?;
    let _bytes_read = file.read(&mut buffer[..]).await?;
    Ok(buffer)
  }

  async fn read_to_writer(
    &mut self,
    _offset: u64,
    _length: u64,
    _buf: &mut (impl async_std::io::Write + Send),
  ) -> Result<(), Self::Error> {
    unimplemented!()
  }

  async fn del(
    &mut self,
    _offset: u64,
    _length: u64,
  ) -> Result<(), Self::Error> {
    panic!("Not implemented yet");
  }

  async fn truncate(&mut self, length: u64) -> Result<(), Self::Error> {
    let file = self.file.as_ref().expect("self.file was None.");
    self.length = length as u64;
    file.set_len(self.length).await?;
    if self.auto_sync {
      file.sync_all().await?;
    }
    Ok(())
  }

  async fn len(&self) -> Result<u64, Self::Error> {
    Ok(self.length)
  }

  async fn is_empty(&mut self) -> Result<bool, Self::Error> {
    Ok(self.length == 0)
  }

  async fn sync_all(&mut self) -> Result<(), Self::Error> {
    if !self.auto_sync {
      let file = self.file.as_ref().expect("self.file was None.");
      file.sync_all().await?;
    }
    Ok(())
  }
}

impl Drop for RandomAccessDisk {
  fn drop(&mut self) {
    if let Some(file) = &self.file {
      // We need to flush the file on drop. Unfortunately, that is not possible to do in a
      // non-blocking fashion, but our only other option here is losing data remaining in the
      // write cache. Good task schedulers should be resilient to occasional blocking hiccups in
      // file destructors so we don't expect this to be a common problem in practice.
      // (from async_std::fs::File::drop)
      let _ = async_std::task::block_on(file.sync_all());
    }
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
  pub fn auto_sync(mut self, auto_sync: bool) -> Self {
    self.auto_sync = auto_sync;
    self
  }

  pub async fn build(self) -> Result<RandomAccessDisk, Error> {
    if let Some(dirname) = self.filename.parent() {
      mkdirp::mkdirp(&dirname)?;
    }
    let file = OpenOptions::new()
      .create(true)
      .read(true)
      .write(true)
      .open(&self.filename)
      .await?;
    file.sync_all().await?;

    let metadata = self.filename.metadata()?;
    Ok(RandomAccessDisk {
      filename: self.filename,
      file: Some(file),
      length: metadata.len(),
      auto_sync: self.auto_sync,
    })
  }
}
