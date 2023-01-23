use anyhow::Error;
#[cfg(feature = "async-std")]
use async_std::fs;
#[cfg(feature = "tokio")]
use tokio::fs;

/// Get file length and file system block size
pub async fn get_length_and_block_size(
  file: &fs::File,
) -> Result<(u64, u64), Error> {
  let metadata = file.metadata().await?;
  Ok((metadata.len(), 0))
}

/// Non-sparse trimming of a file to zeros
pub async fn trim(
  file: &mut fs::File,
  offset: u64,
  length: u64,
  _block_size: u64,
) -> Result<(), Error> {
  #[cfg(feature = "async-std")]
  use async_std::io::{
    prelude::{SeekExt, WriteExt},
    SeekFrom,
  };
  #[cfg(feature = "tokio")]
  use std::io::SeekFrom;
  #[cfg(feature = "tokio")]
  use tokio::io::{AsyncSeekExt, AsyncWriteExt};

  let data = vec![0 as u8; length as usize];
  file.seek(SeekFrom::Start(offset)).await?;
  file.write_all(&data).await?;
  Ok(())
}
