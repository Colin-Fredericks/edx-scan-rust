// Reads tar.gz files and searches for specific regex strings

use std::fs::File;
use std::io::Read;

use clap::Parser;
use flate2::read::GzDecoder;
use regex::Regex;
use tar::{Archive, Entry};

const FILE_TYPES: [&str; 3] = [".html", ".xml", ".json"];

fn main() {
    // Get command-line arguments with clap
    #[derive(Parser)]
    #[clap(
        author = "Colin Fredericks",
        version = "0.1",
        about = "Reads tar.gz files and searches for regex patterns",
        long_about = concat!("This file reads in tarballs from edX ",
            "and searches them for specific text as defined by a regex pattern. ",
            "It includes .html, .xml, and .json files only others.",
            "Run using `cargo run (regex pattern) (tarball)`.")
    )]
    struct CommandLineArgs {
        regex_pattern: String,
        // Handle any number of files, for wildcard purposes
        // TODO: Add in glob so we can handle wildcards on Windows
        tar_gz_path: Vec<String>,
    }
    let args = CommandLineArgs::parse();

    // Print them out to test
    println!("Regex pattern: {}", args.regex_pattern);
    println!("Tar.gz file(s): {:?}", args.tar_gz_path);

    for path in args.tar_gz_path {
        search_in_tarball(&path, &args.regex_pattern);
    }
}

/// Searches the files inside a tarball for a regex pattern.
/// Limits the search to the files in FILE_TYPES.
///
/// # Arguments
/// * **path**: The path to the tarball.
/// * **regex_pattern**: The regex pattern to search for.
fn search_in_tarball(path: &String, regex_pattern: &String) {
    // Open the tar.gz file
    let tarfile_result = File::open(&path);
    let tarfile = match tarfile_result {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            std::process::exit(1);
        }
    };

    // Get all the filenames in the tarball.
    let mut archive_result = Archive::new(GzDecoder::new(tarfile));
    let entries = match archive_result.entries() {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading entries: {}", e);
            std::process::exit(1);
        }
    };
    println!("\nOpened tarball: {}", &path);

    // Iterate through all files in the tarball
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
        // If this is a directory within the tarball, don't bother searching it.
        if entry.header().entry_type().is_dir() {
            continue;
        }
        // Get the file extension
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        // Check if the file is one of the types we want
        if !FILE_TYPES.contains(&extension) {
            continue;
        }

        println!("Found file: {}", path.display());

        // Read the file to one big string
        let contents_str = match read_file_to_string(entry) {
            Ok(contents_str) => contents_str,
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                continue;
            }
        };

        // Compile the regex
        let regex = match Regex::new(regex_pattern) {
            Ok(regex) => regex,
            Err(e) => {
                eprintln!("Error compiling regex: {}", e);
                continue;
            }
        };

        // Search for the regex pattern
        if regex.is_match(&contents_str) {
            println!("  Found match!");
        } else {
            // println!("No match");
        }
    }
}

/// Reads a file from a tarball and returns its contents as a string.
///
/// # Arguments
/// * **entry**: The entry to read from the tarball.
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
    // println!("Contents: {}", contents_str);
    return Ok(contents_str);
}
