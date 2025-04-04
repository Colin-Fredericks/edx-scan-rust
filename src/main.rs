// Reads tar.gz files and searches for specific regex strings

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use tar::Archive;
use flate2::read::GzDecoder;

fn main() {
    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tar.gz file> <regex pattern>", args[0]);
        std::process::exit(1);
    }
    let tar_gz_path = &args[1];
    let regex_pattern = &args[2];

    // Print them out to test
    println!("Tar.gz file: {}", tar_gz_path);
    println!("Regex pattern: {}", regex_pattern);
}
