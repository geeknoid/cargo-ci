use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::Local;

pub struct Log {
    file: File,
}

impl Log {
    pub fn new(workspace_dir: &Path, log_prefix: &str, log_file: Option<&PathBuf>, log_retention_count: usize) -> io::Result<Self> {
        let log_path = if let Some(path) = log_file {
            path.clone()
        } else {
            let log_dir = workspace_dir.join("target").join("logs").join("cargo-ci");
            fs::create_dir_all(&log_dir)?;

            prune_old_logs(&log_dir, log_prefix, log_retention_count);

            let now = Local::now();
            let timestamp = now.format("%Y-%m-%dT%H-%M-%S").to_string();
            log_dir.join(format!("{log_prefix}-{timestamp}.log"))
        };

        let file = OpenOptions::new().create(true).append(true).open(log_path)?;

        Ok(Self { file })
    }

    fn log(&mut self, level: &str, message: impl AsRef<str>) -> io::Result<()> {
        writeln!(self.file, "[{level}] {}", message.as_ref())
    }

    #[expect(clippy::print_stderr, reason = "The point...")]
    pub fn info(&mut self, message: impl AsRef<str>) {
        if let Err(e) = self.log("INFO", message.as_ref()) {
            eprintln!("Failed to write to log file: {e}");
        }
    }

    #[expect(clippy::print_stderr, reason = "The point...")]
    pub fn warn(&mut self, message: impl AsRef<str>) {
        if let Err(e) = self.log("WARN", message.as_ref()) {
            eprintln!("Failed to write to log file: {e}");
        }
    }

    #[expect(clippy::print_stderr, reason = "The point...")]
    pub fn error(&mut self, message: impl AsRef<str>) {
        if let Err(e) = self.log("ERROR", message.as_ref()) {
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
        .filter(|entry| {
            let path = entry.path();
            if !path.is_file() || path.extension() != Some(OsStr::new("log")) {
                return false;
            }
            path.file_name().and_then(|s| s.to_str()).is_some_and(|s| s.starts_with(log_prefix))
        })
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            let modified = meta.modified().ok()?;
            Some((modified, entry.path()))
        })
        .collect();

    // 3. Sort by time (Newest first)
    logs.sort_by(|a, b| b.0.cmp(&a.0));

    // 4. Identify files to delete
    if logs.len() > log_retention_count {
        let to_delete = &logs[log_retention_count..];

        for (_, path) in to_delete {
            _ = fs::remove_file(path);
        }
    }
}
