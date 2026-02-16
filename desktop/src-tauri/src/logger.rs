use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

// Global mutex to ensure thread-safe writing
static LOG_FILE: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Global buffer of log messages for the frontend to poll
static LOG_BUFFER: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

pub fn log(message: &str) {
    log_with_level(message, LogLevel::Info);
}

pub fn debug(message: &str) {
    log_with_level(message, LogLevel::Debug);
}

pub fn log_with_level(message: &str, level: LogLevel) {
    // Only print to stderr if >= Info
    if level >= LogLevel::Info {
        eprintln!("[{}] {}", level, message);
    }

    // Push to in-memory buffer for frontend polling
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        buf.push(format!("[{}] {}", level, message));
        // Cap at 1000 entries to prevent unbounded growth
        if buf.len() > 1000 {
            let drain_count = buf.len() - 1000;
            buf.drain(0..drain_count);
        }
    }

    // Append to file for persistence (all levels)
    let _guard = LOG_FILE.lock().unwrap();
    let log_path = dirs::home_dir()
        .map(|p| p.join(".veteran").join("backend.log"))
        .unwrap_or_else(|| PathBuf::from("backend.log"));
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(file, "[{}] {}", level, message);
    }
}

/// Drain up to `limit` log messages from the buffer (for frontend polling)
pub fn drain_logs(limit: usize) -> Vec<String> {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        let count = buf.len().min(limit);
        buf.drain(0..count).collect()
    } else {
        Vec::new()
    }
}
