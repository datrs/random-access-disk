#![feature(test)]

mod sync {
  extern crate random_access_disk as rad;
  extern crate tempfile;
  extern crate test;

  use self::test::Bencher;

  #[bench]
  fn write_hello_world(b: &mut Bencher) {
    let dir = tempfile::Builder::new()
      .prefix("random-access-disk")
      .tempdir()
      .unwrap();
    let mut file = rad::RandomAccessDisk::new(dir.path().join("1.db"));
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
    let mut file = rad::RandomAccessDisk::new(dir.path().join("2.db"));
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
    let mut file = rad::RandomAccessDisk::new(dir.path().join("3.db"));
    b.iter(|| {
      file.write(0, b"hello").unwrap();
      file.write(5, b" world").unwrap();
      let _text = file.read(0, 11).unwrap();
    });
  }
}
