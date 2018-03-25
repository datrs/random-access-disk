extern crate failure;
extern crate mkdirp;
extern crate random_access_storage as random_access;

use failure::Error;
use std::path;
use std::fs;

/// Main constructor.
pub struct Sync {}

impl Sync {
  /// Create a new instance.
  pub fn new(filename: path::PathBuf) -> random_access::Sync<SyncMethods> {
    random_access::Sync::new(SyncMethods {
      filename: filename,
      fd: None,
    })
  }
}

/// Methods that have been implemented to provide synchronous access to disk.  .
/// These should generally be kept private, but exposed to prevent leaking
/// internals.
pub struct SyncMethods {
  pub filename: path::PathBuf,
  pub fd: Option<fs::File>,
}

impl random_access::SyncMethods for SyncMethods {
  fn open(&mut self) -> Result<(), Error> {
    if let &Some(dirname) = &self.filename.parent() {
      mkdirp::mkdirp(&self.filename)?;
    }

    let fd = fs::File::open(&self.filename)?;
    self.fd = Some(fd);
    Ok(())
  }

  fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
    let fd = self.fd.as_ref().expect("No fd found.");
    Ok(())
  }

  fn read(&mut self, offset: usize, length: usize) -> Result<Vec<u8>, Error> {
    Ok(b"sup".to_vec())
  }

  fn del(&mut self, offset: usize, length: usize) -> Result<(), Error> {
    Ok(())
  }
}
