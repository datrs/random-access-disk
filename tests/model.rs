#[macro_use]
extern crate quickcheck;
extern crate rand;
extern crate random_access_disk as rad;
extern crate random_access_storage;
extern crate tempfile;

use self::Op::*;
use quickcheck::{Arbitrary, Gen};
use rand::Rng;
use random_access_storage::RandomAccess;
use std::u8;
use tempfile::Builder;

const MAX_FILE_SIZE: u64 = 5 * 10; // 5mb

#[derive(Clone, Debug)]
enum Op {
  Read { offset: u64, length: u64 },
  Write { offset: u64, data: Vec<u8> },
}

impl Arbitrary for Op {
  fn arbitrary<G: Gen>(g: &mut G) -> Self {
    let offset: u64 = g.gen_range(0, MAX_FILE_SIZE);
    let length: u64 = g.gen_range(0, MAX_FILE_SIZE / 3);

    if g.gen::<bool>() {
      Read { offset, length }
    } else {
      let mut data = Vec::with_capacity(length as usize);
      for _ in 0..length {
        data.push(u8::arbitrary(g));
      }
      Write { offset, data }
    }
  }
}

quickcheck! {
  fn implementation_matches_model(ops: Vec<Op>) -> bool {
    let dir = Builder::new().prefix("random-access-disk").tempdir().unwrap();

    let mut implementation = rad::RandomAccessDisk::open(dir.path().join("1.db")).unwrap();
    let mut model = vec![];

    for op in ops {
      match op {
        Read { offset, length } => {
          let end = offset + length;
          if model.len() as u64 >= end {
            assert_eq!(
              implementation.read(offset, length).expect("Reads should be successful."),
              &model[offset as usize..end as usize]
            );
          } else {
            assert!(implementation.read(offset, length).is_err());
          }
        },
        Write { offset, ref data } => {
          let end = offset + (data.len() as u64);
          if (model.len() as u64) < end {
            model.resize(end as usize, 0);
          }
          implementation.write(offset, data).expect("Writes should be successful.");
          model[offset as usize..end as usize].copy_from_slice(data);
        },
      }
    }
    true
  }
}
