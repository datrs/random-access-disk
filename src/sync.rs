extern crate failure;
extern crate mkdirp;
extern crate random_access_storage as random_access;

use failure::Error;
use std::path;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};

/// Main constructor.
pub struct Sync {}

impl Sync {
  /// Create a new instance.
  pub fn new(filename: String) -> random_access::Sync<SyncMethods> {
    random_access::Sync::new(SyncMethods {
      filename: path::PathBuf::from(filename),
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

    self.file = Some(fs::File::open(&self.filename)?);
    Ok(())
  }

  fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
    let mut file = self.file.as_ref().expect("self.file was None.");
    file.seek(SeekFrom::Start(offset as u64))?;
    file.write_all(&data)?;
    Ok(())
  }

  fn read(&mut self, offset: usize, length: usize) -> Result<Vec<u8>, Error> {
    let mut file = self.file.as_ref().expect("self.file was None.");
    let mut buffer = Vec::with_capacity(length);
    file.seek(SeekFrom::Start(offset as u64))?;
    file.read(&mut buffer[..])?;
    Ok(buffer)
  }

  fn del(&mut self, _offset: usize, _length: usize) -> Result<(), Error> {
    panic!("Not implemented yet");
  }
}
