use chrono::Local;
use log::LevelFilter;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Holds the current log file and its date label.
struct LogFile {
    file: std::fs::File,
    date: String,
}

/// A simple file logger that writes log entries with daily rotation.
struct FileLogger {
    log_dir: PathBuf,
    state: Mutex<Option<LogFile>>,
}

impl FileLogger {
    fn new(log_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&log_dir);
        let state = Mutex::new(None);
        Self { log_dir, state }
    }

    fn get_or_open_file(&self) -> Option<std::fs::File> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut guard = self.state.lock().ok()?;

        // If we already have a file for today, clone the handle
        if let Some(ref log_file) = *guard {
            if log_file.date == today {
                return log_file.file.try_clone().ok();
            }
        }

        // Open a new file for today
        let path = self.log_dir.join(format!("{}.log", today));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .inspect_err(|e| eprintln!("Failed to open log file {:?}: {}", path, e))
            .ok()?;

        *guard = Some(LogFile {
            date: today,
            file: file.try_clone().ok()?,
        });

        Some(file)
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level = record.level();
        let target = record.target();
        let args = record.args();

        let line = format!("{} [{}] {} - {}\n", timestamp, level, target, args);

        // Write to log file
        if let Some(mut file) = self.get_or_open_file() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }

        // Also write to stderr (visible in dev console)
        eprint!("{}", line);
    }

    fn flush(&self) {
        if let Ok(guard) = self.state.lock() {
            if let Some(ref log_file) = *guard {
                let _ = (&log_file.file).flush();
            }
        }
    }
}

/// Initializes the logging system.
///
/// Creates log files in the `./logs/` directory with daily rotation.
/// Supported log levels: Error, Warn, Info, Debug
pub fn init_logging() {
    let log_dir = PathBuf::from("logs");
    let logger = FileLogger::new(log_dir);

    log::set_max_level(LevelFilter::Debug);
    log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger");

    log::info!("Logging initialized");
}
