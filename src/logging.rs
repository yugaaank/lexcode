use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::db::default_data_dir;

pub fn debug(message: &str) {
    if std::env::var("CODELEX_DEBUG").is_ok() {
        let _ = write_log("debug.log", message);
    }
}

pub fn error(message: &str) {
    let _ = write_log("error.log", message);
}

fn write_log(file: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let directory = default_data_dir()?.join("logs");
    fs::create_dir_all(&directory)?;
    let mut handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join(file))?;
    writeln!(handle, "{message}")?;
    Ok(())
}
