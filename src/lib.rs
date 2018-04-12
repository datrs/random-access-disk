// #![deny(warnings, missing_docs)]
// #![cfg_attr(test, feature(plugin))]
// #![cfg_attr(test, plugin(clippy))]

//! Continuously read,write to disk, using random offsets and lengths.
//!
//! ```rust,no_run
//! # extern crate failure;
//! # extern crate tempdir;
//! # use failure::Error;
//! # fn run_main() -> Result<(), Error> {
//! extern crate tempdir;
//! extern crate random_access_disk;
//!
//! use std::path::PathBuf;
//! use tempdir::TempDir;
//!
//! let dir = TempDir::new("random-access-disk").unwrap();
//! let mut file = random_access_disk::Sync::new(PathBuf::from("./file.log"));
//!
//! file.write(0, b"hello")?;
//! file.write(5, b" world")?;
//! let text = file.read(0, 11)?;
//! # Ok(())
//! # }
//! # fn main() {}
//! ```

#[macro_use]
extern crate failure;

/// Synchronous implementation.
mod sync;

pub use sync::*;
