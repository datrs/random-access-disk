/// OSX-specific trimming of a file to a sparse file
pub fn trim(file: &async_std::fs::File, offset: u64, length: u64) {
  use std::os::unix::io::AsRawFd;
  let fd = file.as_raw_fd();
  println!(
    "TODO: trim fd {} from offset {} to length {}",
    fd, offset, length
  );
}
