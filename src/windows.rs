use anyhow::Error;

pub async fn get_length_and_block_size(
  _file: &async_std::fs::File,
) -> Result<(u64, u64), Error> {
  unimplemented!();
}

/// Windows-specific trimming of a file to a sparse file
pub async fn trim(
  _file: &async_std::fs::File,
  _offset: u64,
  _length: u64,
  _block_size: u64,
) -> Result<(), Error> {
  unimplemented!();
  // See
  // https://github.com/aj-bagwell/drill-press/blob/master/src/windows.rs
  // for inspiration
}
