extern crate failure;
extern crate mkdirp;
extern crate random_access_storage as random_access;

use std::fs;
use failure::Error;
use std::path;

/// Main constructor.
pub struct Sync {}

impl Sync {
  /// Create a new instance.
  pub fn new<'f>(filename: &'f path::Path) -> random_access::Sync<SyncMethods> {
    let methods = SyncMethods { filename: filename };

    random_access::Sync::new(methods)
  }
}

/// Methods that have been implemented to provide synchronous access to disk.  .
/// These should generally be kept private, but exposed to prevent leaking
/// internals.
pub struct SyncMethods<'f> {
  pub filename: &'f path::Path,
  pub fd: fs::File,
}

impl<'f> random_access::SyncMethods for SyncMethods<'f> {
  fn open(&mut self) -> Result<(), Error> {
    if let Some(dirname) = &self.filename.parent() {
      mkdirp::mkdirp(&self.filename)?;
    }
    let fd = fs::open(&self.filename)?;
    self.fd = fd;
    Ok(())
  }

  fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
    Ok(())
  }

  fn read(&mut self, offset: usize, length: usize) -> Result<Vec<u8>, Error> {
    Ok(b"sup".to_vec())
  }

  fn del(&mut self, offset: usize, length: usize) -> Result<(), Error> {
    Ok(())
  }
}
