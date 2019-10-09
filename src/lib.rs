#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(test, deny(warnings))]

use failure::{ensure, Error};
use random_access_storage::RandomAccess;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
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
  pub fn open(filename: path::PathBuf) -> Result<RandomAccessDisk, Error> {
    Self::builder(filename).build()
  }
  pub fn builder(filename: path::PathBuf) -> Builder {
    Builder::new(filename)
  }
}

impl RandomAccess for RandomAccessDisk {
  type Error = Error;

  fn write(&mut self, offset: u64, data: &[u8]) -> Result<(), Self::Error> {
    let mut file = self.file.as_ref().expect("self.file was None.");
    file.seek(SeekFrom::Start(offset))?;
    file.write_all(&data)?;
    if self.auto_sync {
      file.sync_all()?;
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
  fn read(&mut self, offset: u64, length: u64) -> Result<Vec<u8>, Self::Error> {
    ensure!(
      (offset + length) as u64 <= self.length,
      format!(
        "Read bounds exceeded. {} < {}..{}",
        self.length,
        offset,
        offset + length
      )
    );

    let mut file = self.file.as_ref().expect("self.file was None.");
    let mut buffer = vec![0; length as usize];
    file.seek(SeekFrom::Start(offset))?;
    let _bytes_read = file.read(&mut buffer[..])?;
    Ok(buffer)
  }

  fn read_to_writer(
    &mut self,
    _offset: u64,
    _length: u64,
    _buf: &mut impl Write,
  ) -> Result<(), Self::Error> {
    unimplemented!()
  }

  fn del(&mut self, _offset: u64, _length: u64) -> Result<(), Self::Error> {
    panic!("Not implemented yet");
  }

  fn truncate(&mut self, length: u64) -> Result<(), Self::Error> {
    let file = self.file.as_ref().expect("self.file was None.");
    self.length = length as u64;
    file.set_len(self.length)?;
    if self.auto_sync {
      file.sync_all()?;
    }
    Ok(())
  }

  fn len(&self) -> Result<u64, Self::Error> {
    Ok(self.length)
  }

  fn is_empty(&mut self) -> Result<bool, Self::Error> {
    Ok(self.length == 0)
  }

  fn sync_all(&mut self) -> Result<(), Self::Error> {
    if !self.auto_sync {
      let file = self.file.as_ref().expect("self.file was None.");
      file.sync_all()?;
    }
    Ok(())
  }
}

impl Drop for RandomAccessDisk {
  fn drop(&mut self) {
    if let Some(file) = &self.file {
      file.sync_all().unwrap();
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
  pub fn build(self) -> Result<RandomAccessDisk, Error> {
    if let Some(dirname) = self.filename.parent() {
      mkdirp::mkdirp(&dirname)?;
    }
    let file = OpenOptions::new()
      .create(true)
      .read(true)
      .write(true)
      .open(&self.filename)?;
    file.sync_all()?;

    let metadata = self.filename.metadata()?;
    Ok(RandomAccessDisk {
      filename: self.filename,
      file: Some(file),
      length: metadata.len(),
      auto_sync: self.auto_sync,
    })
  }
}
