# edx-scan-rust

Performs regex searches on edX course tarballs
without unzipping the tarball.

## Installation

[Install Rust first](https://www.rust-lang.org/tools/install).

Clone the repo and `cd` into it.

```bash
> cargo build
> cargo run test/zippy.tgz "regex_pattern"
```
