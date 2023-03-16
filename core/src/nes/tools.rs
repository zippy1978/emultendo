use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

/// Loads execution trace from file.
pub fn load_trace<P>(file: P) -> Result<Vec<String>, String>
where
    P: AsRef<Path>,
{
    let file = File::open(file).map_err(|e| e.to_string())?;
    let buf = BufReader::new(file);
    Ok(buf.lines().map(|l| l.unwrap()).collect())
}
