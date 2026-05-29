# edx-scan-rust

Performs regex searches on edX course tarballs
without unzipping the tarball.

## Installation

[Install Rust first](https://www.rust-lang.org/tools/install).

Clone the repo and `cd` into it.

```bash
> cargo build
> cargo run "regex_pattern" test/zippy.tgz
```

## Nice-to-have

This currently works as a command-line utility, returning just filenames and pathnames and printing them to the terminal. Potential expansions:

- Print to a CSV
- Include file locations and names
- Include extra context - lines before and after the one that was matched.
- Nicer interface than command line