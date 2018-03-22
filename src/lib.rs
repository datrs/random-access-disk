// #![deny(warnings, missing_docs)]
#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(clippy))]

//! Continuously read,write to disk, using random offsets and lengths.

// #[macro_use]
extern crate failure;

/// Synchronous implementation.
mod sync;

pub use sync::*;
