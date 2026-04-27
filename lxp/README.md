# Build
cargo build --release

# Run — redirect stdout to a file
./target/release/tarball-json-search "some pattern" data/*.tar.gz > results.csv

# Case-insensitive search (regex flag)
./target/release/tarball-json-search "(?i)error" archive.tar.gz > results.csv