## 2020-02-02, Version 1.0.0
### Commits
- [[`3de8d37a0c`](https://github.com/datrs/random-access-disk/commit/3de8d37a0c33bc15874de8870734bbdd07a8b24c)] 1.0.0 (substack)
- [[`2fca3428a6`](https://github.com/datrs/random-access-disk/commit/2fca3428a6ca6e484c5b31c8b62422553641731e)] Merge pull request #28 from bltavares/bump-deps (Szabolcs Berecz)
- [[`bebe8b4a2c`](https://github.com/datrs/random-access-disk/commit/bebe8b4a2c95dcd6d19c54ceb2fe854799af4ea4)] Update code to editio 2018 (Bruno Tavares)
- [[`310dd8a5d7`](https://github.com/datrs/random-access-disk/commit/310dd8a5d79e2edcfbded8cb48a17c5949c6b276)] Bump random-access-storage to 3.0.0 (Bruno Tavares)
- [[`95c3dad271`](https://github.com/datrs/random-access-disk/commit/95c3dad2715ade75c18337bb72f109573c9c08e3)] updated to use u64 (substack)
- [[`1d884a4432`](https://github.com/datrs/random-access-disk/commit/1d884a443232528c2a0a9e03901cb12eca137ac1)] sync_all implementation (substack)
- [[`bd9d0f6a86`](https://github.com/datrs/random-access-disk/commit/bd9d0f6a869d9f8225ae8582a5bfe5db6ca52578)] Update mkdirp requirement from 0.1.0 to 1.0.0 (dependabot-preview[bot])
- [[`f9b34735e7`](https://github.com/datrs/random-access-disk/commit/f9b34735e79020ff2f39bef510026f5cf8bf02b3)] Update quickcheck requirement from 0.7.2 to 0.8.0 (dependabot[bot])
- [[`4785a3dfa5`](https://github.com/datrs/random-access-disk/commit/4785a3dfa5aed164df833eafaeef4edda0dfd2c4)] Update changelog (Yoshua Wuyts)

### Stats
```diff
 CHANGELOG.md                       |  16 ++++++-
 Cargo.toml                         |  25 ++++-----
 fuzz/Cargo.toml                    |   1 +-
 fuzz/fuzz_targets/fuzz_target_1.rs |   6 +--
 src/lib.rs                         | 109 ++++++++++++++++++++++++--------------
 tests/model.rs                     |  34 ++++--------
 tests/regression.rs                |   5 +--
 tests/test.rs                      |  82 +++++++++++++++++++++++++++--
 8 files changed, 194 insertions(+), 84 deletions(-)
```


## 2018-12-20, Version 0.8.0
### Commits
- [[`c7cdc2d39a`](https://github.com/datrs/random-access-disk/commit/c7cdc2d39a7d92103f7125c28d2451ecf6272578)] (cargo-release) version 0.8.0 (Yoshua Wuyts)
- [[`65bffdf0b8`](https://github.com/datrs/random-access-disk/commit/65bffdf0b8ef6d7b37a89b9ef498fb62a24a2e25)] len() and is_empty() implementation (#21) (James Halliday)
- [[`e6733d9273`](https://github.com/datrs/random-access-disk/commit/e6733d92734e42881ca5f90228eef96883b005a7)] Update changelog (Yoshua Wuyts)

### Stats
```diff
 CHANGELOG.md  | 22 ++++++++++++++++++++++
 Cargo.toml    |  4 ++--
 src/lib.rs    |  8 ++++++++
 tests/test.rs | 38 ++++++++++++++++++++++++++++++++++++++
 4 files changed, 70 insertions(+), 2 deletions(-)
```


## 2018-11-21, Version 0.7.0
### Commits
- [[`9059d4f552`](https://github.com/datrs/random-access-disk/commit/9059d4f5524f16f52badff98a92e4b7db308a2d0)] (cargo-release) version 0.7.0 (Yoshua Wuyts)
- [[`791e3fd8ee`](https://github.com/datrs/random-access-disk/commit/791e3fd8ee1fd7af387119e734fd498074fc8c33)] update travis (Yoshua Wuyts)
- [[`4e33df9813`](https://github.com/datrs/random-access-disk/commit/4e33df981357474f1e986e3fef63f5b4397efad0)] truncate implementation with tests (#18) (James Halliday)
- [[`4098c2d35d`](https://github.com/datrs/random-access-disk/commit/4098c2d35dd570d635a890656925ae898e8e9e05)] Update rand requirement from 0.5.5 to 0.6.0 (#17) (dependabot[bot])
- [[`9083d3cfb0`](https://github.com/datrs/random-access-disk/commit/9083d3cfb08069541a146f7e12e6af06c97354c0)] Update quickcheck requirement from 0.6.2 to 0.7.1 (#15) (dependabot[bot])
- [[`928fe1afaf`](https://github.com/datrs/random-access-disk/commit/928fe1afaf95453a00bb7754bb7ff91e08fe5689)] Run clippy on travis (#14) (Szabolcs Berecz)
- [[`07fa83dd28`](https://github.com/datrs/random-access-disk/commit/07fa83dd2882d0b6868378e7a9599572693e035e)] update changelog (Yoshua Wuyts)

### Stats
```diff
 .travis.yml    | 23 ++++++++++---------
 CHANGELOG.md   | 21 ++++++++++++++++++-
 Cargo.toml     | 11 ++++-----
 src/lib.rs     | 10 +++++++-
 tests/model.rs |  2 ++-
 tests/test.rs  | 70 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 6 files changed, 121 insertions(+), 16 deletions(-)
```


## 2018-08-30, Version 0.6.0
### Commits
- [[`1070eb3166`](https://github.com/datrs/random-access-disk/commits/1070eb31665c3578842997557af292a9e702a033)] (cargo-release) version 0.6.0 (Yoshua Wuyts)
- [[`fb9ee81c81`](https://github.com/datrs/random-access-disk/commits/fb9ee81c81043619ecf6ea3a5d670373248cd677)] Random access always open (#13) (Szabolcs Berecz)
- [[`0a18b10972`](https://github.com/datrs/random-access-disk/commits/0a18b109722c73f7385f77fe7fb7c2d118f7bcae)] replace tempdir crate (deprecated) with tempfile crate, using tempfile::Builder to create a tempdir (#12) (Jacob Burden)
- [[`254d3ccf77`](https://github.com/datrs/random-access-disk/commits/254d3ccf7789e615a46815c0e43f0892aab96eff)] update changelog (Yoshua Wuyts)

### Stats
```diff
 .travis.yml         |  1 +-
 CHANGELOG.md        | 26 ++++++++++++++++++++++++-
 Cargo.toml          |  6 ++---
 benches/sync.rs     | 29 +++++++++++++++++++--------
 src/lib.rs          | 59 +++++++++++++++++++++++++-----------------------------
 tests/model.rs      | 10 +++++----
 tests/regression.rs | 16 ++++++++++-----
 tests/test.rs       | 35 +++++++++++++++++++++-----------
 8 files changed, 120 insertions(+), 62 deletions(-)
```


## 2018-08-23, Version 0.5.0
### Commits
- [[`647536ba06`](https://github.com/datrs/random-access-disk/commits/647536ba06ab55f810c7981e60d68481ec55044c)] (cargo-release) version 0.5.0 (Yoshua Wuyts)
- [[`556d70f09a`](https://github.com/datrs/random-access-disk/commits/556d70f09a0b23cf15107442f9cefec7669ad463)] upgrade random-access-storage (#9)
- [[`61af2acc13`](https://github.com/datrs/random-access-disk/commits/61af2acc135456d39eb05b92e1ad3a20e790e53c)] Fix typo in crates.io link (#10)
 (tomasol)
- [[`64f674e8e9`](https://github.com/datrs/random-access-disk/commits/64f674e8e9b7377b209775e5bf31238f6be213cb)] Rename Sync -> RandomAccessDisk in README.md (#11)
 (tomasol)
- [[`1860e0ce4d`](https://github.com/datrs/random-access-disk/commits/1860e0ce4d8b0de8fce189beaaad549d79b3d40f)] rm unused src/syc file (Yoshua Wuyts)
- [[`e5089b73ff`](https://github.com/datrs/random-access-disk/commits/e5089b73ffc2a75210fa2c2fab52ee0050486ec6)] fix rustfmt in travis.yml (Yoshua Wuyts)
- [[`7a4448f454`](https://github.com/datrs/random-access-disk/commits/7a4448f454bcc57f158d6c360a5d82727a6a74e9)] remove &* calls (Yoshua Wuyts)
- [[`522cd4219e`](https://github.com/datrs/random-access-disk/commits/522cd4219e8bfd37cb3403f1100d6024f5367f2b)] (cargo-release) start next development iteration 0.4.1-alpha.0 (Yoshua Wuyts)

### Stats
```diff
 .travis.yml    |   2 +-
 Cargo.toml     |   4 +-
 README.md      |   4 +-
 src/lib.rs     |  26 +++++++++-----
 src/sync.rs    | 105 +----------------------------------------------------------
 tests/model.rs |   5 +--
 tests/test.rs  |   5 +---
 7 files changed, 26 insertions(+), 125 deletions(-)
```


