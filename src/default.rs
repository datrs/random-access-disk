use anyhow::Error;

/// Get file length and file system block size
pub async fn get_length_and_block_size(
  file: &async_std::fs::File,
) -> Result<(u64, u64), Error> {
  let metadata = file.metadata().await?;
  Ok((metadata.len(), 0))
}

/// Non-sparse trimming of a file to zeros
pub async fn trim(
  file: &mut async_std::fs::File,
  offset: u64,
  length: u64,
  _block_size: u64,
) -> Result<(), Error> {
  use async_std::io::prelude::{SeekExt, WriteExt};
  use async_std::io::SeekFrom;
  let data = vec![0 as u8; length as usize];
  file.seek(SeekFrom::Start(offset)).await?;
  file.write_all(&data).await?;
  Ok(())
}
