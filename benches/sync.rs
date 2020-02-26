#![feature(test)]

mod sync {
  extern crate test;
  use random_access_disk as rad;
  use test::Bencher;

  use random_access_storage::RandomAccess;

  #[bench]
  fn write_hello_world(b: &mut Bencher) {
    async_std::task::block_on(async {
      let dir = tempfile::Builder::new()
        .prefix("random-access-disk")
        .tempdir()
        .unwrap();
      let mut file = rad::RandomAccessDisk::open(dir.path().join("1.db"))
        .await
        .unwrap();
      b.iter(|| {
        async_std::task::block_on(async {
          file.write(0, b"hello").await.unwrap();
          file.write(5, b" world").await.unwrap();
        })
      });
    });
  }

  #[bench]
  fn read_hello_world(b: &mut Bencher) {
    async_std::task::block_on(async {
      let dir = tempfile::Builder::new()
        .prefix("random-access-disk")
        .tempdir()
        .unwrap();
      let mut file = rad::RandomAccessDisk::open(dir.path().join("2.db"))
        .await
        .unwrap();
      file.write(0, b"hello").await.unwrap();
      file.write(5, b" world").await.unwrap();
      b.iter(|| {
        async_std::task::block_on(async {
          let _text = file.read(0, 11).await.unwrap();
        })
      });
    });
  }

  #[bench]
  fn read_write_hello_world(b: &mut Bencher) {
    async_std::task::block_on(async {
      let dir = tempfile::Builder::new()
        .prefix("random-access-disk")
        .tempdir()
        .unwrap();
      let mut file = rad::RandomAccessDisk::open(dir.path().join("3.db"))
        .await
        .unwrap();
      b.iter(|| {
        async_std::task::block_on(async {
          file.write(0, b"hello").await.unwrap();
          file.write(5, b" world").await.unwrap();
          let _text = file.read(0, 11).await.unwrap();
        })
      });
    });
  }
}
