extern crate failure;
extern crate mkdirp;
extern crate random_access_storage as random_access;

use failure::Error;
use std::path;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

/// Main constructor.
pub struct Sync {}

impl Sync {
  /// Create a new instance.
  pub fn new(filename: path::PathBuf) -> random_access::Sync<SyncMethods> {
    random_access::Sync::new(SyncMethods {
      filename: filename,
      file: None,
    })
  }
}

/// Methods that have been implemented to provide synchronous access to disk.  .
/// These should generally be kept private, but exposed to prevent leaking
/// internals.
pub struct SyncMethods {
  pub filename: path::PathBuf,
  pub file: Option<fs::File>,
}

impl random_access::SyncMethods for SyncMethods {
  fn open(&mut self) -> Result<(), Error> {
    if let &Some(dirname) = &self.filename.parent() {
      mkdirp::mkdirp(&dirname)?;
    }
    self.file = Some(OpenOptions::new()
      .create_new(true)
      .read(true)
      .write(true)
      .open(&self.filename)?);
    Ok(())
  }

  fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
    let mut file = self.file.as_ref().expect("self.file was None.");
    file.seek(SeekFrom::Start(offset as u64))?;
    file.write(&data)?;
    Ok(())
  }

  fn read(&mut self, offset: usize, length: usize) -> Result<Vec<u8>, Error> {
    let mut file = self.file.as_ref().expect("self.file was None.");
    let mut buffer = Vec::with_capacity(length);
    for _ in 0..length {
      buffer.push(0);
    }
    file.seek(SeekFrom::Start(offset as u64))?;
    file.read(&mut buffer[..])?;
    println!("{:?}", offset);
    Ok(buffer)
  }

  fn del(&mut self, _offset: usize, _length: usize) -> Result<(), Error> {
    panic!("Not implemented yet");
  }
}
