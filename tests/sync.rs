mod sync {
  extern crate random_access_disk as rad;
  extern crate tempdir;

  use self::tempdir::TempDir;

  #[test]
  fn can_call_new() {
    let dir = TempDir::new("random-access-disk").unwrap();
    let _file = rad::Sync::new(dir.path().join("1.db"));
  }

  #[test]
  fn can_open_buffer() {
    let dir = TempDir::new("random-access-disk").unwrap();
    let mut file = rad::Sync::new(dir.path().join("2.db"));
    file.write(0, b"hello").unwrap();
    assert!(file.opened);
  }

  #[test]
  fn can_write() {
    let dir = TempDir::new("random-access-disk").unwrap();
    let mut file = rad::Sync::new(dir.path().join("3.db"));
    file.write(0, b"hello").unwrap();
    file.write(5, b" world").unwrap();
  }

  #[test]
  fn can_read() {
    let dir = TempDir::new("random-access-disk").unwrap();
    let mut file = rad::Sync::new(dir.path().join("4.db"));
    file.write(0, b"hello").unwrap();
    file.write(5, b" world").unwrap();
    let text = file.read(0, 11).unwrap();
    assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello world");
  }
}
