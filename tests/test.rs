use random_access_disk as rad;
use random_access_storage::RandomAccess;
use std::io::Read;
use tempfile::Builder;

#[async_std::test]
async fn can_call_new() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let _file = rad::RandomAccessDisk::open(dir.path().join("1.db"))
    .await
    .unwrap();
}

#[async_std::test]
async fn can_open_buffer() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("2.db"))
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
}

#[async_std::test]
async fn can_write() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("3.db"))
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
}

#[async_std::test]
async fn can_read() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("4.db"))
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  let text = file.read(0, 11).await.unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello world");
}

#[async_std::test]
async fn can_truncate_lt() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("5.db"))
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  file.truncate(7).await.unwrap();
  let text = file.read(0, 7).await.unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello w");
  match file.read(0, 8).await {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("5.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello w");
}

#[async_std::test]
async fn can_truncate_gt() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("6.db"))
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  file.truncate(15).await.unwrap();
  let text = file.read(0, 15).await.unwrap();
  assert_eq!(
    String::from_utf8(text.to_vec()).unwrap(),
    "hello world\0\0\0\0"
  );
  match file.read(0, 16).await {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("6.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello world\0\0\0\0");
}

#[async_std::test]
async fn can_truncate_eq() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("7.db"))
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  file.truncate(11).await.unwrap();
  let text = file.read(0, 11).await.unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello world");
  match file.read(0, 12).await {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("7.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello world");
}

#[async_std::test]
async fn can_del_short() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::builder(dir.path().join("13.db"))
    .auto_sync(true)
    .build()
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  file.write(11, b" people").await.unwrap();
  file.del(5, 6).await.unwrap();
  let hello = file.read(0, 5).await.unwrap();
  assert_eq!(String::from_utf8(hello.to_vec()).unwrap(), "hello");
  let zeros = file.read(5, 6).await.unwrap();
  assert_eq!(zeros, vec![0; 6]);
  let people = file.read(12, 6).await.unwrap();
  assert_eq!(String::from_utf8(people.to_vec()).unwrap(), "people");
}

#[async_std::test]
async fn can_del_long() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::builder(dir.path().join("14.db"))
    .auto_sync(true)
    .build()
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  const MULTI_BLOCK_LEN: usize = 4096 * 3;
  let multi_block = &[0x61 as u8; MULTI_BLOCK_LEN];
  file.write(5, multi_block).await.unwrap();
  file
    .write((MULTI_BLOCK_LEN + 5) as u64, b"people")
    .await
    .unwrap();
  file.del(5, MULTI_BLOCK_LEN as u64).await.unwrap();
  let hello = file.read(0, 5).await.unwrap();
  assert_eq!(String::from_utf8(hello.to_vec()).unwrap(), "hello");
  let zeros = file.read(5, 10).await.unwrap();
  assert_eq!(zeros, vec![0; 10]);
  let zeros = file.read(MULTI_BLOCK_LEN as u64, 5).await.unwrap();
  assert_eq!(zeros, vec![0; 5]);
  let zeros = file.read((MULTI_BLOCK_LEN / 2) as u64, 5).await.unwrap();
  assert_eq!(zeros, vec![0; 5]);
  let people = file.read((MULTI_BLOCK_LEN + 5) as u64, 6).await.unwrap();
  assert_eq!(String::from_utf8(people.to_vec()).unwrap(), "people");
}

#[async_std::test]
async fn can_len() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("8.db"))
    .await
    .unwrap();
  assert_eq!(file.len().await.unwrap(), 0);
  file.write(0, b"hello").await.unwrap();
  assert_eq!(file.len().await.unwrap(), 5);
  file.write(5, b" world").await.unwrap();
  assert_eq!(file.len().await.unwrap(), 11);
  file.truncate(15).await.unwrap();
  assert_eq!(file.len().await.unwrap(), 15);
  file.truncate(8).await.unwrap();
  assert_eq!(file.len().await.unwrap(), 8);
}

#[async_std::test]
async fn can_is_empty() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::open(dir.path().join("9.db"))
    .await
    .unwrap();
  assert_eq!(file.is_empty().await.unwrap(), true);
  file.write(0, b"hello").await.unwrap();
  assert_eq!(file.is_empty().await.unwrap(), false);
  file.truncate(0).await.unwrap();
  assert_eq!(file.is_empty().await.unwrap(), true);
  file.truncate(1).await.unwrap();
  assert_eq!(file.is_empty().await.unwrap(), false);
  file.truncate(0).await.unwrap();
  assert_eq!(file.is_empty().await.unwrap(), true);
  file.write(0, b"what").await.unwrap();
  assert_eq!(file.is_empty().await.unwrap(), false);
}

#[async_std::test]
async fn explicit_no_auto_sync() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::builder(dir.path().join("10.db"))
    .auto_sync(false)
    .build()
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  file.truncate(11).await.unwrap();
  file.sync_all().await.unwrap();
  let text = file.read(0, 11).await.unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello world");
  match file.read(0, 12).await {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("10.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello world");
}

#[async_std::test]
async fn explicit_auto_sync() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::builder(dir.path().join("11.db"))
    .auto_sync(true)
    .build()
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  file.truncate(11).await.unwrap();
  let text = file.read(0, 11).await.unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello world");
  match file.read(0, 12).await {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("11.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello world");
}

#[async_std::test]
async fn explicit_auto_sync_with_sync_call() {
  let dir = Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  let mut file = rad::RandomAccessDisk::builder(dir.path().join("12.db"))
    .auto_sync(true)
    .build()
    .await
    .unwrap();
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  file.truncate(11).await.unwrap();
  file.sync_all().await.unwrap();
  let text = file.read(0, 11).await.unwrap();
  assert_eq!(String::from_utf8(text.to_vec()).unwrap(), "hello world");
  match file.read(0, 12).await {
    Ok(_) => panic!("file is too big. read past the end should have failed"),
    _ => {}
  };
  let mut c_file = std::fs::File::open(dir.path().join("12.db")).unwrap();
  let mut c_contents = String::new();
  c_file.read_to_string(&mut c_contents).unwrap();
  assert_eq!(c_contents, "hello world");
}
