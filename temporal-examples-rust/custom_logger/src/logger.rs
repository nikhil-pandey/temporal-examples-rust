//! Custom logger implementation that prints human-readable logs to stdout (via
//! `env_logger`) **and** writes JSON-encoded log records to `worker.log`.
//!
//! The JSON lines are one-per-log-record with the following shape:
//! `{ "ts": "2025-07-03T18:25:17Z", "level": "INFO", "target": "my_target", "msg": "hello" }`
//!
//! The logger is **only meant for example/demo purposes** – it is **not** an
//! ergonomic or high-performance production logger. The important part is to
//! show that the Temporal Rust worker can install *any* logger that implements
//! the [`log::Log`] trait.

use std::{
    fs::OpenOptions,
    io::Write,
    sync::{Mutex, OnceLock},
};

use chrono::Utc;
use env_logger::{Builder, Env};
use log::{LevelFilter, Log, Metadata, Record};

/// Install the custom logger as the global `log` implementation.
///
/// Calling this function more than once has no effect – only the first call
/// registers the logger (subsequent calls are ignored).
pub fn init() {
    static START: OnceLock<()> = OnceLock::new();
    START.get_or_init(|| {
        // stdout logger with env-controlled filter (fallback `info`).
        let env_logger = Builder::from_env(Env::default().default_filter_or("info")).build();

        // File sink (append) – create if missing.
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("worker.log")
            .expect("failed opening worker.log for append");

        let logger = CombinedLogger {
            stdout: env_logger,
            file: Mutex::new(file),
        };

        // Safety: we only call once above.
        log::set_boxed_logger(Box::new(logger)).expect("failed to set global logger");
        log::set_max_level(LevelFilter::Info);
    });
}

/// Logger that multiplexes to stdout (via `env_logger::Logger`) and the log file.
struct CombinedLogger {
    stdout: env_logger::Logger,
    file: Mutex<std::fs::File>,
}

impl Log for CombinedLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.stdout.enabled(metadata)
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // 1. Human-readable stdout via env_logger.
        self.stdout.log(record);

        // 2. JSON line to the file.
        if let Ok(mut file) = self.file.lock() {
            // Build simple JSON – avoid extra dependencies.
            let ts = Utc::now().to_rfc3339();
            // Escape quotes/backslashes in message manually (very minimal – good
            // enough for demo).
            let msg = record.args().to_string().replace('\"', "\\\"");
            let target = record.target();
            let level = record.level();
            // Write directly without temporary variable to satisfy `uninlined_format_args`.
            #[allow(clippy::uninlined_format_args)]
            let _ = writeln!(
                file,
                "{{\"ts\":\"{}\",\"level\":\"{}\",\"target\":\"{}\",\"msg\":\"{}\"}}",
                ts, level, target, msg
            );
        }
    }

    fn flush(&self) {
        let _ = self.file.lock().map(|mut f| f.flush());
        self.stdout.flush();
    }
}
