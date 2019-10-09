#![no_main]

use libfuzzer_sys::{fuzz_target};
use random_access_disk as rad;
use self::tempdir::TempDir;

fuzz_target!(|data: &[u8]| {
  let dir = TempDir::new("random-access-disk").unwrap();
  let mut file = rad::Sync::new(dir.path().join("2.db"));
  file.write(0, data).unwrap();
});
