#![feature(test)]

mod sync {
  extern crate random_access_disk as rad;
  extern crate random_access_storage;
  extern crate tempfile;
  extern crate test;

  use self::random_access_storage::RandomAccess;
  use self::test::Bencher;

  #[bench]
  fn write_hello_world(b: &mut Bencher) {
    let dir = tempfile::Builder::new()
      .prefix("random-access-disk")
      .tempdir()
      .unwrap();
    let mut file =
      rad::RandomAccessDisk::open(dir.path().join("1.db")).unwrap();
    b.iter(|| {
      file.write(0, b"hello").unwrap();
      file.write(5, b" world").unwrap();
    });
  }

  #[bench]
  fn read_hello_world(b: &mut Bencher) {
    let dir = tempfile::Builder::new()
      .prefix("random-access-disk")
      .tempdir()
      .unwrap();
    let mut file =
      rad::RandomAccessDisk::open(dir.path().join("2.db")).unwrap();
    file.write(0, b"hello").unwrap();
    file.write(5, b" world").unwrap();
    b.iter(|| {
      let _text = file.read(0, 11).unwrap();
    });
  }

  #[bench]
  fn read_write_hello_world(b: &mut Bencher) {
    let dir = tempfile::Builder::new()
      .prefix("random-access-disk")
      .tempdir()
      .unwrap();
    let mut file =
      rad::RandomAccessDisk::open(dir.path().join("3.db")).unwrap();
    b.iter(|| {
      file.write(0, b"hello").unwrap();
      file.write(5, b" world").unwrap();
      let _text = file.read(0, 11).unwrap();
    });
  }
}
