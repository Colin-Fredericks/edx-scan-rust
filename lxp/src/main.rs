//! Search for a regular expression across all JSON files embedded in `.tar.gz` archives.
//! No files are extracted to disk; everything is handled in memory.
//!
//! # Usage
//! ```
//! cargo run -- <regex_pattern> <archive.tar.gz> [archive2.tar.gz ...]  > results.csv
//! ```
//!
//! # Output columns
//! tarball | json_file | line_number | match_text | type | activity_id | parent_id | title | description

use std::{
    env,
    error::Error,
    fs::File,
    io::{self, BufReader, Read, Write},
};

use csv::Writer;
use flate2::read::GzDecoder;
use regex::Regex;
use serde_json::Value;
use tar::Archive;

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "Usage: {} <regex_pattern> <archive.tar.gz> [archive2.tar.gz ...]\n\
             Results are written as CSV to stdout.",
            args[0]
        );
        std::process::exit(1);
    }

    let pattern = &args[1];
    let tarball_paths = &args[2..];

    let regex = Regex::new(pattern)
        .map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;

    let mut wtr = Writer::from_writer(io::stdout());

    wtr.write_record(&[
        "tarball",
        "json_file",
        "line_number",
        "match_text",
        "type",
        "activity_id",
        "parent_id",
        "title",
        "description",
    ])?;

    for path in tarball_paths {
        if let Err(e) = process_tarball(path, &regex, &mut wtr) {
            eprintln!("Error processing tarball '{}': {}", path, e);
        }
    }

    wtr.flush()?;
    Ok(())
}

// ── Tarball processing ───────────────────────────────────────────────────────

fn process_tarball<W: Write>(
    tarball_path: &str,
    regex: &Regex,
    wtr: &mut Writer<W>,
) -> Result<(), Box<dyn Error>> {
    let file = File::open(tarball_path)?;
    let gz = GzDecoder::new(BufReader::new(file));
    let mut archive = Archive::new(gz);

    for entry_result in archive.entries()? {
        let mut entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("  [{}] Skipping unreadable entry: {}", tarball_path, e);
                continue;
            }
        };

        let path_buf = match entry.path() {
            Ok(p) => p.to_path_buf(),
            Err(e) => {
                eprintln!(
                    "  [{}] Skipping entry with unreadable path: {}",
                    tarball_path, e
                );
                continue;
            }
        };

        let json_path = path_buf.to_string_lossy().into_owned();

        if !json_path.ends_with(".json") {
            continue;
        }

        // Read the entire JSON file into memory as a UTF-8 string.
        let mut content = String::new();
        if let Err(e) = entry.read_to_string(&mut content) {
            eprintln!("  [{}] Cannot read '{}': {}", tarball_path, json_path, e);
            continue;
        }

        if let Err(e) = process_json_file(tarball_path, &json_path, &content, regex, wtr) {
            eprintln!(
                "  [{}] Error processing '{}': {}",
                tarball_path, json_path, e
            );
        }
    }

    Ok(())
}

// ── JSON file processing ─────────────────────────────────────────────────────

fn process_json_file<W: Write>(
    tarball: &str,
    json_path: &str,
    content: &str,
    regex: &Regex,
    wtr: &mut Writer<W>,
) -> Result<(), Box<dyn Error>> {
    // Build a byte-offset → 1-based line-number lookup table.
    let line_table = build_line_table(content);

    // Treat the raw file content as a plain string for the regex search
    // (this covers both keys and values in the JSON).
    let matches: Vec<(usize, &str)> = regex
        .find_iter(content)
        .map(|m| (m.start(), m.as_str()))
        .collect();

    if matches.is_empty() {
        return Ok(());
    }

    // Parse the JSON as an array of objects for structured field extraction.
    let parsed: Option<Vec<Value>> = match serde_json::from_str(content) {
        Ok(Value::Array(arr)) => Some(arr),
        Ok(_) => {
            eprintln!(
                "  [{}] '{}' is not a JSON array — structured fields will be empty.",
                tarball, json_path
            );
            None
        }
        Err(e) => {
            eprintln!(
                "  [{}] JSON parse error in '{}': {} — structured fields will be empty.",
                tarball, json_path, e
            );
            None
        }
    };

    // Compute the byte span of every top-level object so we can map a match
    // position back to the correct index in `parsed`.
    let obj_ranges: Vec<(usize, usize)> = if parsed.is_some() {
        find_object_byte_ranges(content)
    } else {
        Vec::new()
    };

    for (byte_offset, match_text) in &matches {
        let line_num = line_table.get(*byte_offset).copied().unwrap_or(1);

        let (type_val, activity_id, parent_id, title, description) =
            fields_for_offset(*byte_offset, &obj_ranges, parsed.as_deref());

        wtr.write_record(&[
            tarball,
            json_path,
            &line_num.to_string(),
            match_text,
            &type_val,
            &activity_id,
            &parent_id,
            &title,
            &description,
        ])?;
    }

    Ok(())
}

// ── Field extraction ─────────────────────────────────────────────────────────

/// Find the top-level JSON object whose byte span contains `byte_offset`
/// and return its known fields. Returns empty strings when no match is found.
fn fields_for_offset(
    byte_offset: usize,
    obj_ranges: &[(usize, usize)],
    parsed: Option<&[Value]>,
) -> (String, String, String, String, String) {
    let arr = match parsed {
        Some(a) => a,
        None => return default_fields(),
    };

    let idx = obj_ranges
        .iter()
        .position(|(s, e)| byte_offset >= *s && byte_offset <= *e);

    match idx.and_then(|i| arr.get(i)) {
        Some(obj) => extract_fields(obj),
        None => default_fields(),
    }
}

/// Extract the five known fields from a parsed JSON object.
/// Any absent or wrongly-typed field is represented as an empty string.
fn extract_fields(obj: &Value) -> (String, String, String, String, String) {
    let type_val = obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let activity_id = obj
        .get("activity_id")
        .and_then(|v| v.as_i64())
        .map(|n| n.to_string())
        .unwrap_or_default();

    let parent_id = obj
        .get("parent_id")
        .and_then(|v| v.as_i64())
        .map(|n| n.to_string())
        .unwrap_or_default();

    let data = obj.get("data");

    let title = data
        .and_then(|d| d.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let description = data
        .and_then(|d| d.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    (type_val, activity_id, parent_id, title, description)
}

fn default_fields() -> (String, String, String, String, String) {
    (
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    )
}

// ── Low-level text utilities ─────────────────────────────────────────────────

/// Build a lookup table where `table[i]` is the 1-based line number of byte `i`.
/// An extra sentinel entry is appended for positions equal to `content.len()`.
fn build_line_table(content: &str) -> Vec<usize> {
    let mut table = Vec::with_capacity(content.len() + 1);
    let mut line = 1usize;
    for &b in content.as_bytes() {
        table.push(line);
        if b == b'\n' {
            line += 1;
        }
    }
    table.push(line); // sentinel for end-of-content
    table
}

/// Scan `content` byte-by-byte and return the inclusive byte range `(start, end)`
/// of every **top-level** object inside the outermost JSON array.
/// `start` points at the `{` byte and `end` points at the matching `}` byte.
///
/// Depth semantics during the scan:
/// - 0  → outside everything
/// - 1  → inside the outermost `[…]`
/// - 2  → inside a direct-child `{…}` (the objects we want)   ← tracked
/// - 3+ → nested deeper inside a top-level object
///
/// Because every structurally significant character (`[`, `]`, `{`, `}`, `"`, `\`)
/// is plain ASCII, byte-by-byte scanning is correct for arbitrary UTF-8 content.
fn find_object_byte_ranges(content: &str) -> Vec<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut obj_start: usize = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        // The byte immediately after a `\` is always a literal — skip it.
        if escape_next {
            escape_next = false;
            continue;
        }

        if in_string {
            match byte {
                b'\\' => escape_next = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }

        // Structural bytes outside string literals:
        match byte {
            b'"' => in_string = true,

            // Arrays increase depth but don't start a tracked object.
            b'[' => depth += 1,

            // An object opening at depth 1 (direct child of the root array)
            // is the start of a top-level object we want to track.
            b'{' => {
                depth += 1;
                if depth == 2 {
                    obj_start = i;
                }
            }

            b']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }

            // A closing `}` at depth 2 finishes a top-level object.
            b'}' => {
                if depth == 2 {
                    ranges.push((obj_start, i));
                }
                if depth > 0 {
                    depth -= 1;
                }
            }

            _ => {}
        }
    }

    ranges
}