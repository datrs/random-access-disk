extern crate random_access_disk as rad;
extern crate tempdir;

use self::tempdir::TempDir;
use std::env;

#[test]
// postmortem: read_exact wasn't behaving like we hoped, so we had to switch
// back to `.read()` and disable clippy for that rule specifically.
fn regress_1() {
  let dir = TempDir::new("random-access-disk").unwrap();
  let mut file = rad::Sync::new(dir.path().join("regression-1.db"));
  file.write(27, b"").unwrap();
  file.read(13, 5).unwrap();
}

#[test]
// postmortem: accessing the same file twice would fail, so we had to switch to
// from `.create_new()` to `.create()`.
//
// NOTE: test needs to be run twice in a row to trigger regression. I'm sorry.
fn regress_2() {
  let mut dir = env::temp_dir();
  dir.push("regression-2.db");
  let mut file = rad::Sync::new(dir);
  file.write(27, b"").unwrap();
}
