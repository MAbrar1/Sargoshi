//! Loader for the generated known-answer vector files in `vectors/`. Each non-comment line is a
//! record of space-separated fields; `-` stands for an empty value.
#![forbid(unsafe_code)]

use std::path::PathBuf;

/// Read every record of a vector file, skipping comment lines.
///
/// # Errors
/// Returns a message if the file cannot be read.
pub fn load(name: &str) -> Result<Vec<Vec<String>>, String> {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "vectors", name]
        .iter()
        .collect();
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split(' ').map(str::to_owned).collect())
        .collect())
}

/// Decode a lowercase or uppercase hex string; `-` decodes to the empty byte string.
///
/// # Errors
/// Returns a message for an odd length or a non-hex character.
pub fn hex(s: &str) -> Result<Vec<u8>, String> {
    if s == "-" {
        return Ok(Vec::new());
    }
    if !s.len().is_multiple_of(2) {
        return Err(format!("odd hex length: {s}"));
    }
    s.as_bytes()
        .chunks(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|e| e.to_string())?;
            u8::from_str_radix(text, 16).map_err(|e| format!("{e}: {text}"))
        })
        .collect()
}

/// Take field `i` of a record, or fail with a clear message.
///
/// # Errors
/// Returns a message if the record has fewer than `i + 1` fields.
pub fn field(record: &[String], i: usize) -> Result<&str, String> {
    record
        .get(i)
        .map(String::as_str)
        .ok_or_else(|| format!("record has no field {i}"))
}
