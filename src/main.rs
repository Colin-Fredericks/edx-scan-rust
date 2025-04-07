// Reads tar.gz files and searches for specific regex strings

use flate2::read::GzDecoder;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use tar::Archive;
use clap::Parser;

fn main() {
    // Get command-line arguments with clap
    // Should be `cargo run (tarball) (regex pattern)`
    #[derive(Parser)]
    #[clap(author = "Colin Fredericks", version = "0.1", about = "Reads tar.gz files and searches for regex patterns")]
    struct Cli {
        /// Path to the tar.gz file
        #[arg(short, long)]
        tar_gz_path: String,
        /// Regex pattern to search for
        #[arg(short, long)]
        regex_pattern: String,
    }
    let args = Cli::parse();

    // Print them out to test
    println!("Tar.gz file: {}", args.tar_gz_path);
    println!("Regex pattern: {}", args.regex_pattern);

    // Open the tar.gz file
    let tarfile_result = File::open(args.tar_gz_path);
    let tarfile = match tarfile_result {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            std::process::exit(1);
        }
    };

    // Print out all the filenames in the tarball.
    let mut archive_result = Archive::new(GzDecoder::new(tarfile));
    let entries = match archive_result.entries() {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading entries: {}", e);
            std::process::exit(1);
        }
    };

    for item in entries {
        let mut entry = match item {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
                continue;
            }
        };
        let path = match entry.path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error getting path: {}", e);
                continue;
            }
        };
        println!("Found file: {}", path.display());

        // Read the contents of the file
        let mut contents = Vec::new();
        if let Err(e) = entry.read_to_end(&mut contents) {
            eprintln!("Error reading file: {}", e);
            continue;
        }
        // Convert contents to a string
        let contents_str = match String::from_utf8(contents) {
            Ok(string) => string,
            Err(e) => {
                eprintln!("Error converting to string: {}", e);
                continue;
            }
        };
        // Print the contents
        println!("Contents: {}", contents_str);

        // Search for the regex pattern
        let regex = match Regex::new(&args.regex_pattern) {
            Ok(regex) => regex,
            Err(e) => {
                eprintln!("Error compiling regex: {}", e);
                continue;
            }
        };
        if regex.is_match(&contents_str) {
            println!("Found match!");
        } else {
            println!("No match");
        }
    }
}
