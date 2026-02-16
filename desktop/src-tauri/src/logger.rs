use std::fs::OpenOptions;
use std::io::Write;
use std::sync::LazyLock;
use std::sync::Mutex;

// Global mutex to ensure thread-safe writing
static LOG_FILE: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Global buffer of log messages for the frontend to poll
static LOG_BUFFER: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn log(message: &str) {
    // Print to stderr for terminal visibility
    eprintln!("{}", message);

    // Push to in-memory buffer for frontend polling
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        buf.push(message.to_string());
        // Cap at 1000 entries to prevent unbounded growth
        if buf.len() > 1000 {
            let drain_count = buf.len() - 1000;
            buf.drain(0..drain_count);
        }
    }

    // Append to file for persistence
    let _guard = LOG_FILE.lock().unwrap();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/milanleonard/Projects/VRPRookieCloneWorking/backend.log")
    {
        let _ = writeln!(file, "{}", message);
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
