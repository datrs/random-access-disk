extern crate random_access_disk as rad;
extern crate tempdir;

use self::tempdir::TempDir;

#[test]
// postmortem: read_exact wasn't behaving like we hoped, so we had to switch
// back to `.read()` and disable clippy for that rule specifically.
fn regress_1() {
  let dir = TempDir::new("random-access-disk").unwrap();
  let mut file = rad::Sync::new(dir.path().join("regression-1.db"));
  file.write(27, b"").unwrap();
  file.read(13, 5).unwrap();
}
