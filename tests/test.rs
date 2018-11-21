extern crate random_access_disk as rad;
extern crate random_access_storage;
extern crate tempfile;

use random_access_storage::RandomAccess;
use std::io::Read;
use tempfile::Builder;

#[test]
fn can_call_new() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let _file = rad::RandomAccessDisk::open(dir.path().join("1.db")).unwrap();
}

#[test]
fn can_open_buffer() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("2.db")).unwrap();
  file.write(0, b"hello").unwrap();
}

#[test]
fn can_write() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("3.db")).unwrap();
  file.write(0, b"hello").unwrap();
  file.write(5, b" world").unwrap();
}

#[test]
fn can_read() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("4.db")).unwrap();
  file.write(0, b"hello").unwrap();
  file.write(5, b" world").unwrap();
  let text = file.read(0, 11).unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello world");
}

#[test]
fn can_truncate_lt() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("5.db")).unwrap();
  file.write(0, b"hello").unwrap();
  file.write(5, b" world").unwrap();
  file.truncate(7).unwrap();
  let text = file.read(0, 7).unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello w");
  match file.read(0, 8) {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("5.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello w");
}

#[test]
fn can_truncate_gt() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("6.db")).unwrap();
  file.write(0, b"hello").unwrap();
  file.write(5, b" world").unwrap();
  file.truncate(15).unwrap();
  let text = file.read(0, 15).unwrap();
  assert_eq!(
    String::from_utf8(text.to_vec()).unwrap(),
    "hello world\0\0\0\0"
  );
  match file.read(0, 16) {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("6.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello world\0\0\0\0");
}

#[test]
fn can_truncate_eq() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("7.db")).unwrap();
  file.write(0, b"hello").unwrap();
  file.write(5, b" world").unwrap();
  file.truncate(11).unwrap();
  let text = file.read(0, 11).unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello world");
  match file.read(0, 12) {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("7.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello world");
}
