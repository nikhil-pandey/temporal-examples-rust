//! Worker sets up custom logger (stdout + file, JSON lines) and runs Temporal worker.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

use env_logger::{Builder, Env};
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use custom_logger::activities::log_activity;
use custom_logger::workflow::logging_workflow;

use once_cell::sync::Lazy;

/// Global for the worker log file path (so all modules know it).
pub static LOGFILE_PATH: &str = "worker.log";
// File writer wrapped in Mutex so log formatter closure can append.
static FILE_WRITER: Lazy<Mutex<Option<std::fs::File>>> = Lazy::new(|| {
    let f = OpenOptions::new().create(true).append(true).write(true).open(LOGFILE_PATH).ok();
    Mutex::new(f)
});

fn init_custom_logger() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            // Message to be logged: emit to stdout in default format
            let ts = buf.timestamp_custom("%Y-%m-%d %H:%M:%S", buf.local_time());
            let msg = format!("{ts} [{lvl}] {target}: {body}\n",
                ts=ts,
                lvl=record.level(),
                target=record.target(),
                body=record.args()
            );
            // JSON-one-line to file (info up)
            if record.level() <= log::Level::Info {
                // Compose a JSON line and write it out
                // Example: {"ts": ..., "level":..., "target":..., "msg":...}
                let json = format!(
                    "{{\"ts\":\"{}\",\"level\":\"{}\",\"target\":\"{}\",\"msg\":\"{}\"}}\n",
                    ts,
                    record.level(),
                    record.target(),
                    escape_json_string(&format!("{}", record.args()))
                );
                if let Ok(mut filelock) = FILE_WRITER.lock() {
                    if let Some(f) = filelock.as_mut() {
                        let _ = f.write_all(json.as_bytes());
                    }
                }
            }
            // Always write stack format to stdout
            std::io::Write::write_all(buf, msg.as_bytes())?;
            Ok(())
        })
        .init();
}

/// Helper: escape for JSON string value
fn escape_json_string(s: &str) -> String {
    s.replace('\"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_custom_logger();
    info!(target: "custom_logger", "Custom logger worker initializing ...");

    let client = get_client().await?;
    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;
    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("custom-logger")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;
    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "custom-logger");

    worker.register_activity("log_activity", log_activity);
    worker.register_wf("logging_workflow", logging_workflow);
    worker.run().await?;
    Ok(())
}
