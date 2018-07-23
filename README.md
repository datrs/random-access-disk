# random-access-disk
[![crates.io version][1]][2] [![build status][3]][4]
[![downloads][5]][6] [![docs.rs docs][7]][8]

Continuously read,write to disk, using random offsets and lengths. Adapted from
[random-access-storage/random-access-file](https://github.com/random-access-storage/random-access-file/).

- [Documentation][8]
- [Crate][2]

## Usage
```rust
extern crate tempdir;
extern crate random_access_disk;

use std::path::PathBuf;
use tempdir::TempDir;

let dir = TempDir::new("random-access-disk").unwrap();
let mut file = random_access_disk::Sync::new(dir.path().join("README.db"));

file.write(0, b"hello").unwrap();
file.write(5, b" world").unwrap();
let _text = file.read(0, 11).unwrap();
```

## Installation
```sh
$ cargo add random-access-disk
```

## License
[MIT](./LICENSE-MIT) OR [Apache-2.0](./LICENSE-APACHE)

[1]: https://img.shields.io/crates/v/random-access-disk.svg?style=flat-square
[2]: https://crates.io/crates/random-access-disk
[3]: https://img.shields.io/travis/datrs/random-access-disk.svg?style=flat-square
[4]: https://travis-ci.org/datrs/random-access-disk
[5]: https://img.shields.io/crates/d/random-access-disk.svg?style=flat-square
[6]: https://crates.io/crates/random-access-disk
[7]: https://docs.rs/random-access-disk/badge.svg
[8]: https://docs.rs/random-access-disk
