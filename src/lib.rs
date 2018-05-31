#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate failure;
extern crate mkdirp;
extern crate random_access_storage;

use random_access_storage::{RandomAccessMethods, RandomAccess};
use failure::Error;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Drop;
use std::path;

/// Main constructor.
#[derive(Debug)]
pub struct RandomAccessDisk {}

impl RandomAccessDisk {
  /// Create a new instance.
  // #[cfg_attr(test, allow(new_ret_no_self))]
  pub fn new(filename: path::PathBuf) -> RandomAccess<RandomAccessDiskMethods> {
    RandomAccess::new(RandomAccessDiskMethods {
      filename,
      file: None,
      length: 0,
    })
  }
}

/// Methods that have been implemented to provide synchronous access to disk.  .
/// These should generally be kept private, but exposed to prevent leaking
/// internals.
#[derive(Debug)]
pub struct RandomAccessDiskMethods {
  filename: path::PathBuf,
  file: Option<fs::File>,
  length: u64,
}

impl RandomAccessMethods for RandomAccessDiskMethods {
  fn open(&mut self) -> Result<(), Error> {
    if let Some(dirname) = self.filename.parent() {
      mkdirp::mkdirp(&dirname)?;
    }
    let file = OpenOptions::new()
      .create(true)
      .read(true)
      .write(true)
      .open(&self.filename)?;
    file.sync_all()?;

    self.file = Some(file);

    let metadata = fs::metadata(&self.filename)?;
    self.length = metadata.len();
    Ok(())
  }

  fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
    let mut file = self.file.as_ref().expect("self.file was None.");
    file.seek(SeekFrom::Start(offset as u64))?;
    file.write_all(&data)?;
    file.sync_all()?;

    // We've changed the length of our file.
    let new_len = (offset + data.len()) as u64;
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
  fn read(&mut self, offset: usize, length: usize) -> Result<Vec<u8>, Error> {
    ensure!(
      (offset + length) as u64 <= self.length,
      format!("Read bounds exceeded. {} < {}..{}",
              self.length, offset, offset + length)
    );

    let mut file = self.file.as_ref().expect("self.file was None.");
    let mut buffer = vec![0; length];
    file.seek(SeekFrom::Start(offset as u64))?;
    file.read(&mut buffer[..])?;
    Ok(buffer)
  }

  fn del(&mut self, _offset: usize, _length: usize) -> Result<(), Error> {
    panic!("Not implemented yet");
  }
}

impl Drop for RandomAccessDiskMethods {
  fn drop(&mut self) {
    if let Some(file) = &self.file {
      file.sync_all().unwrap();
    }
  }
}

