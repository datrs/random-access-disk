// #![deny(warnings, missing_docs)]
// #![cfg_attr(test, feature(plugin))]
// #![cfg_attr(test, plugin(clippy))]

//! Continuously read,write to disk, using random offsets and lengths.
//!
//! ```rust,ignore
//! extern crate random_access_disk;
//!
//! use std::path::PathBuf;
//!
//! let dir = TempDir::new("random-access-disk").unwrap();
//! let mut file = rad::Sync::new(PathBuf::from("./file.log"));
//!
//! file.write(0, b"hello")?;
//! file.write(5, b" world")?;
//! let text = file.read(0, 11)?;
//! ```

// #[macro_use]
extern crate failure;

/// Synchronous implementation.
mod sync;

pub use sync::*;
