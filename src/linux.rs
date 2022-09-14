/// Linux-specific trimming to sparse files
pub fn trim(file: &mut async_std::fs::File, offset: u64, size_t: u64) {
  use std::os::unix::io::AsRawFd;
  let fd = file.as_raw_fd();
  println!(
    "TODO: trim fd {} from offset {} to length {}",
    fd, offset, length
  );
}
