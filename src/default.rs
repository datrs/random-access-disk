use random_access_storage::RandomAccessError;
#[cfg(feature = "async-std")]
use async_std::fs;
#[cfg(feature = "tokio")]
use tokio::fs;

/// Get file length and file system block size
pub async fn get_length_and_block_size(
  file: &fs::File,
) -> Result<(u64, u64), RandomAccessError> {
  let metadata = file.metadata().await?;
  Ok((metadata.len(), 0))
}

/// Set file to sparse, not applicable
pub async fn set_sparse(_file: &mut fs::File) -> Result<(), RandomAccessError> {
  Ok(())
}

/// Non-sparse trimming of a file to zeros
pub async fn trim(
  file: &mut fs::File,
  offset: u64,
  length: u64,
  _block_size: u64,
) -> Result<(), RandomAccessError> {
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
