// Reads tar.gz files and searches for specific regex strings

use clap::Parser;
use flate2::read::GzDecoder;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use tar::{Archive, Entry};

fn main() {
    // Get command-line arguments with clap
    // Should be `cargo run (tarball) (regex pattern)`
    #[derive(Parser)]
    #[clap(author = "Colin Fredericks", version = "0.1", about = "Reads tar.gz files and searches for regex patterns")]
    struct Cli {
        tar_gz_path: String,
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
        // Error check for entry
        let entry = match item {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
                continue;
            }
        };
        // Error check for getting the path of the entry
        let path = match entry.path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error getting path: {}", e);
                continue;
            }
        };
        println!("\nFound file: {}", path.display());

        // Read the file to one big string
        let contents_str = match read_file_to_string(entry) {
            Ok(contents_str) => contents_str,
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                continue;
            }
        };

        // Compile the regex
        let regex = match Regex::new(&args.regex_pattern) {
            Ok(regex) => regex,
            Err(e) => {
                eprintln!("Error compiling regex: {}", e);
                continue;
            }
        };
        // Search for the regex pattern
        if regex.is_match(&contents_str) {
            println!("Found match!\n");
        } else {
            println!("No match");
        }
    }
}

fn read_file_to_string(mut entry: Entry<GzDecoder<File>>) -> Result<String, std::io::Error> {
    // Read the contents of the file
    let mut contents = Vec::new();
    if let Err(e) = entry.read_to_end(&mut contents) {
        eprintln!("Error reading file: {:?}", e);
        return Err(e);
    }
    // Convert contents to a string
    let contents_str = match String::from_utf8(contents) {
        Ok(string) => string,
        Err(e) => {
            eprintln!("Error converting to string: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to convert contents to string",
            ));
        }
    };
    // Print the contents
    println!("Contents: {}", contents_str);
    return Ok(contents_str);
}
