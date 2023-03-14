use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use random_access_disk as rad;
use random_access_storage::RandomAccess;

#[cfg(feature = "async-std")]
use criterion::async_executor::AsyncStdExecutor;

fn bench_write_hello_world(c: &mut Criterion) {
  #[cfg(feature = "async-std")]
  c.bench_function("write hello world", |b| {
    b.to_async(AsyncStdExecutor)
      .iter_custom(|iters| write_hello_world(iters));
  });
  #[cfg(feature = "tokio")]
  c.bench_function("write hello world", |b| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    b.to_async(&rt)
      .iter_custom(|iters| write_hello_world(iters));
  });
}

async fn write_hello_world(iters: u64) -> Duration {
  let mut file = create_file("1.db").await;
  let start = Instant::now();
  for _ in 0..iters {
    black_box(
      async {
        file.write(0, b"hello").await.unwrap();
        file.write(5, b" world").await.unwrap();
      }
      .await,
    );
  }
  start.elapsed()
}

fn bench_read_hello_world(c: &mut Criterion) {
  #[cfg(feature = "async-std")]
  c.bench_function("read hello world", |b| {
    b.to_async(AsyncStdExecutor)
      .iter_custom(|iters| read_hello_world(iters));
  });
  #[cfg(feature = "tokio")]
  c.bench_function("read hello world", |b| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    b.to_async(&rt).iter_custom(|iters| read_hello_world(iters));
  });
}

async fn read_hello_world(iters: u64) -> Duration {
  let mut file = create_file("2.db").await;
  file.write(0, b"hello").await.unwrap();
  file.write(5, b" world").await.unwrap();
  let start = Instant::now();
  for _ in 0..iters {
    black_box(
      async {
        let _text = file.read(0, 11).await.unwrap();
      }
      .await,
    );
  }
  start.elapsed()
}

fn bench_read_write_hello_world(c: &mut Criterion) {
  #[cfg(feature = "async-std")]
  c.bench_function("read/write hello world", |b| {
    b.to_async(AsyncStdExecutor)
      .iter_custom(|iters| read_write_hello_world(iters));
  });
  #[cfg(feature = "tokio")]
  c.bench_function("read/write hello world", |b| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    b.to_async(&rt)
      .iter_custom(|iters| read_write_hello_world(iters));
  });
}

async fn read_write_hello_world(iters: u64) -> Duration {
  let mut file = create_file("3.db").await;
  let start = Instant::now();
  for _ in 0..iters {
    black_box(
      async {
        file.write(0, b"hello").await.unwrap();
        file.write(5, b" world").await.unwrap();
        let _text = file.read(0, 11).await.unwrap();
      }
      .await,
    );
  }
  start.elapsed()
}

fn bench_write_del_hello_world(c: &mut Criterion) {
  #[cfg(feature = "async-std")]
  c.bench_function("write/del hello world", |b| {
    b.to_async(AsyncStdExecutor)
      .iter_custom(|iters| write_del_hello_world(iters));
  });
  #[cfg(feature = "tokio")]
  c.bench_function("write/del hello world", |b| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    b.to_async(&rt)
      .iter_custom(|iters| write_del_hello_world(iters));
  });
}

async fn write_del_hello_world(iters: u64) -> Duration {
  let mut file = create_file("4.db").await;
  let start = Instant::now();
  for _ in 0..iters {
    black_box(
      async {
        file.write(0, b"hello world").await.unwrap();
        file.del(0, 5).await.unwrap();
        file.del(5, 6).await.unwrap();
      }
      .await,
    );
  }
  start.elapsed()
}

async fn create_file(file_name: &str) -> rad::RandomAccessDisk {
  let dir = tempfile::Builder::new()
    .prefix("random-access-disk")
    .tempdir()
    .unwrap();
  rad::RandomAccessDisk::open(dir.path().join(file_name))
    .await
    .unwrap()
}

criterion_group!(
  benches,
  bench_write_hello_world,
  bench_read_hello_world,
  bench_read_write_hello_world,
  bench_write_del_hello_world,
);
criterion_main!(benches);
