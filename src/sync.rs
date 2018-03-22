extern crate failure;
extern crate random_access_storage as random_access;

use failure::Error;

/// Main constructor.
pub struct Sync;
impl Sync {
  /// Create a new instance.
  pub fn new() -> random_access::Sync<SyncMethods> {
    let methods = SyncMethods {};

    random_access::Sync::new(methods)
  }
}

/// Methods that have been implemented to provide synchronous access to disk.  .
/// These should generally be kept private, but exposed to prevent leaking
/// internals.
pub struct SyncMethods {}

impl random_access::SyncMethods for SyncMethods {
  fn open(&mut self) -> Result<(), Error> {
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
