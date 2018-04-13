#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate random_access_disk as rad;
extern crate tempdir;

use self::tempdir::TempDir;

fuzz_target!(|data: &[u8]| {
  let dir = TempDir::new("random-access-disk").unwrap();
  let mut file = rad::Sync::new(dir.path().join("2.db"));
  file.write(0, data).unwrap();
});
