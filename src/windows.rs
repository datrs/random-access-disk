use std::os::windows::prelude::{AsRawHandle, RawHandle};

use anyhow::Error;
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::FSCTL_SET_SPARSE;
use winapi::um::winioctl::FSCTL_SET_ZERO_DATA;

#[cfg(feature = "async-std")]
use async_std::fs;
#[cfg(feature = "tokio")]
use tokio::fs;

pub async fn get_length_and_block_size(
  file: &fs::File,
) -> Result<(u64, u64), Error> {
  let meta = file.metadata().await?;
  Ok((meta.len(), 0))
}

/// Set file to sparse
pub async fn set_sparse(file: &mut fs::File) -> Result<(), Error> {
  unsafe {
    device_io_control(
      file.as_raw_handle(),
      FSCTL_SET_SPARSE,
      &None::<Option<()>>,
      std::ptr::null_mut() as *mut (),
      0,
    )?;
  };

  Ok(())
}

/// Windows-specific trimming of a file to a sparse file
pub async fn trim(
  file: &fs::File,
  offset: u64,
  length: u64,
  _block_size: u64,
) -> Result<(), Error> {
  unsafe {
    device_io_control(
      file.as_raw_handle(),
      FSCTL_SET_ZERO_DATA,
      &FileZeroDataInformation {
        offset,
        beyond_final_zero: offset + length,
      },
      std::ptr::null_mut() as *mut (),
      0,
    )?;
  };
  Ok(())
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FileZeroDataInformation {
  offset: u64,
  beyond_final_zero: u64,
}

unsafe fn device_io_control<Q: Sized, R: Sized>(
  handle: RawHandle,
  control_code: DWORD,
  query: &Q,
  result: *mut R,
  capacity: usize,
) -> Result<usize, std::io::Error> {
  let mut returned_bytes: DWORD = 0;

  let ret = DeviceIoControl(
    handle as _,
    control_code,
    query as *const _ as LPVOID,
    std::mem::size_of::<Q>() as DWORD,
    result as LPVOID,
    capacity as DWORD,
    &mut returned_bytes,
    std::ptr::null_mut(),
  );

  if ret == 0 {
    return Err(std::io::Error::last_os_error().into());
  }

  Ok(returned_bytes as usize)
}
