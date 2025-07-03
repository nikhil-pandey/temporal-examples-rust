//! Binary that starts (or resumes) the `lock_workflow` coordinator for a given
//! lock id. Run this **once** before launching any `client` binaries.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;
use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;

/// Task queue shared with mutex worker.
const TASK_QUEUE: &str = "mutex";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Optional CLI arg – lock id. Defaults to "mutex-lock" for demo.
    let lock_id = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "mutex-lock".to_string());

    let client = get_client().await?;

    info!(target: "mutex", "Starting lock_workflow for lock_id={lock_id}");

    // The coordinator workflow doesn't need input, but Temporal requires at
    // least an empty payload vector.
    let input = vec![json!("").as_json_payload()?];

    // Start the workflow if it doesn't already exist. If another instance with
    // the same workflow id is already running this will return an error, which
    // we treat as a no-op (it just means the coordinator is already active).
    let start_res = client
        .start_workflow(
            input,
            TASK_QUEUE.to_string(),
            lock_id.clone(),
            "lock_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await;

    match start_res {
        Ok(ok) => {
            println!("Started lock_workflow: id={} run_id={}", lock_id, ok.run_id);
        }
        Err(e) => {
            println!("Could not start lock_workflow – likely already running: {e}");
        }
    }

    Ok(())
}
