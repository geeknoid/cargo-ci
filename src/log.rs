use core::cell::RefCell;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::Local;

pub struct Log {
    file: RefCell<BufWriter<File>>,
}

impl Log {
    pub fn new(target_dir: &Path, log_prefix: &str, log_file: Option<&Path>, log_retention_count: usize) -> io::Result<Self> {
        let log_path = if let Some(path) = log_file {
            path.to_path_buf()
        } else {
            let log_dir = target_dir.join("logs").join("cargo-ci");
            fs::create_dir_all(&log_dir)?;

            prune_old_logs(&log_dir, log_prefix, log_retention_count);

            let now = Local::now();
            let timestamp = now.format("%Y-%m-%dT%H-%M-%S").to_string();
            log_dir.join(format!("{log_prefix}-{timestamp}.log"))
        };

        let file = OpenOptions::new().create(true).append(true).open(log_path)?;

        Ok(Self {
            file: RefCell::new(BufWriter::new(file)),
        })
    }

    fn log(&self, level: &str, message: impl AsRef<str>) -> io::Result<()> {
        let mut file = self.file.borrow_mut();
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "[{timestamp}] [{level}] {}", message.as_ref())
    }

    #[expect(clippy::print_stderr, reason = "The point...")]
    pub fn info(&self, message: impl AsRef<str>) {
        if let Err(e) = self.log("INFO", &message) {
            eprintln!("Failed to write to log file: {e}");
        }
    }

    #[expect(clippy::print_stderr, reason = "The point...")]
    pub fn warn(&self, message: impl AsRef<str>) {
        if let Err(e) = self.log("WARN", &message) {
            eprintln!("Failed to write to log file: {e}");
        }
    }

    #[expect(clippy::print_stderr, reason = "The point...")]
    pub fn error(&self, message: impl AsRef<str>) {
        if let Err(e) = self.log("ERROR", &message) {
            eprintln!("Failed to write to log file: {e}");
        }
    }
}

/// Keeps only the N most recent log files in the given directory.
fn prune_old_logs(log_dir: &Path, log_prefix: &str, log_retention_count: usize) {
    let Ok(entries) = fs::read_dir(log_dir) else {
        // Directory probably doesn't exist yet, which is fine.
        return;
    };

    let mut logs: Vec<(SystemTime, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();

            // Check if it's a log file with the right prefix
            if !path.is_file() || path.extension() != Some(OsStr::new("log")) {
                return None;
            }

            let file_name = path.file_name()?.to_str()?;
            if !file_name.starts_with(log_prefix) {
                return None;
            }

            // Get modification time
            let meta = entry.metadata().ok()?;
            let modified = meta.modified().ok()?;

            Some((modified, path))
        })
        .collect();

    // Sort by time (newest first)
    logs.sort_unstable_by(|a, b| b.0.cmp(&a.0));

    // Delete old log files beyond retention count
    if logs.len() > log_retention_count {
        for (_, path) in &logs[log_retention_count..] {
            _ = fs::remove_file(path);
        }
    }
}
