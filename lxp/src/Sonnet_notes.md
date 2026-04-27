# Notes from Sonnet 4.6

| Step | Detail |
|------|--------|
| Archive reading | flate2 decompresses the gzip stream; tar iterates entries — nothing is written to disk. |
| Regex search | The raw file bytes are decoded to a UTF-8 string and searched with regex::Regex::find_iter, so both keys and values are matched. |
| Line numbers | A pre-built Vec\<usize\> maps every byte offset to its 1-based line number in O(1) per lookup. |
| Object correlation | A second byte-by-byte pass (find_object_byte_ranges) records the {…} byte spans of every top-level array element. The match's byte offset is looked up against those spans to select the right parsed Value. |
| Field extraction | serde_json parses the array once; extract_fields walks the Value tree with graceful None-handling for absent keys. |
| Output | The csv crate writes RFC 4180-compliant rows to stdout (quotes and escapes special characters automatically). |
