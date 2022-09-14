use anyhow::{anyhow, Error};

/// Get unix file length and file system block size
pub async fn get_length_and_block_size(
  file: &async_std::fs::File,
) -> Result<(u64, u64), Error> {
  use std::os::unix::fs::MetadataExt;
  let meta = file.metadata().await?;
  let block_size = meta.blksize();
  Ok((meta.len(), block_size))
}

/// Linux-specific trimming to sparse files
#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
pub async fn trim(
  file: &mut async_std::fs::File,
  offset: u64,
  length: u64,
  _block_size: u64,
) -> Result<(), Error> {
  use libc::{fallocate, FALLOC_FL_KEEP_SIZE, FALLOC_FL_PUNCH_HOLE};
  use std::os::unix::io::AsRawFd;

  let fd = file.as_raw_fd();
  unsafe {
    let ret = fallocate(
      fd,
      FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE,
      offset as libc::off_t,
      length as libc::off_t,
    );

    if ret < 0 {
      return Err(anyhow!(
        "Failed to punch hole to file on linux with return {} and OS error {}",
        ret,
        std::io::Error::last_os_error().to_string()
      ));
    }
  }

  Ok(())
}

/// OSX-specific trimming of a file to a sparse file
#[cfg(target_os = "macos")]
pub async fn trim(
  file: &mut async_std::fs::File,
  offset: u64,
  length: u64,
  block_size: u64,
) -> Result<(), Error> {
  use async_std::io::prelude::{SeekExt, WriteExt};
  use async_std::io::SeekFrom;

  let end = offset + length;

  // Find distance to next block
  let next_block_distance: u64 =
    block_size - ((offset + block_size) % block_size);
  if next_block_distance > 0 {
    // Needs zeroing of the current block
    let data = vec![0 as u8; next_block_distance as usize];
    file.seek(SeekFrom::Start(offset)).await?;
    file.write_all(&data).await?;
  }

  // Find offset to from end to the last block
  let last_block_offset = end - (end % block_size);

  // Now see if there are blocks in the middle that can be punched
  // into holes
  let punch_hole_offset = offset + next_block_distance;

  if punch_hole_offset < last_block_offset {
    // There is a t least one block that can be punched
    punch_hole(
      file,
      punch_hole_offset,
      last_block_offset - punch_hole_offset,
    )?;
  }

  if last_block_offset < end {
    // Needs zeroing of the last block
    let data = vec![0 as u8; (end - last_block_offset) as usize];
    file.seek(SeekFrom::Start(last_block_offset)).await?;
    file.write_all(&data).await?;
  }

  Ok(())
}

/// OSX-specific punching of a hole to a file. Works only with offset and length
/// that matches file system block boundaries.
#[cfg(target_os = "macos")]
fn punch_hole(
  file: &async_std::fs::File,
  offset: u64,
  length: u64,
) -> Result<(), Error> {
  // fcntl.h has this, which is not yet covered by libc:
  //
  // #define F_PUNCHHOLE 99 /* Deallocate a range of the file */
  //
  // /* fpunchhole_t used by F_PUNCHHOLE */
  // typedef struct fpunchhole {
  // 	unsigned int fp_flags; /* unused */
  // 	unsigned int reserved; /* (to maintain 8-byte alignment) */
  // 	off_t fp_offset; /* IN: start of the region */
  // 	off_t fp_length; /* IN: size of the region */
  // } fpunchhole_t;
  //
  //  F_PUNCHHOLE  Deallocate a region and replace it with a hole.
  //  Subsequent reads of the affected region will return bytes of
  //  zeros that are usually not backed by physical blocks. This will
  //  not change the actual file size. Holes must be aligned to file
  //  system block boundaries. This will fail on file systems that do
  //  not support this interface.

  use libc::c_int;
  use std::os::unix::io::AsRawFd;

  let fd = file.as_raw_fd();

  #[repr(C)]
  struct fpunchhole_t {
    fp_flags: c_int, /* unused */
    reserved: c_int, /* (to maintain 8-byte alignment) */
    fp_offset: u64,  /* IN: start of the region */
    fp_length: u64,  /* IN: size of the region */
  }
  const F_PUNCHHOLE: c_int = 99;

  let hole = fpunchhole_t {
    fp_flags: 0,
    reserved: 0,
    fp_offset: offset,
    fp_length: length,
  };

  unsafe {
    let ret = libc::fcntl(fd, F_PUNCHHOLE, &hole);
    if ret < 0 {
      return Err(anyhow!(
        "Failed to punch hole to file on macos with return {} and OS error {}",
        ret,
        std::io::Error::last_os_error().to_string()
      ));
    }
  }

  Ok(())
}
