//! Client that starts the cron workflow with a schedule.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;
use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = get_client().await?;

    // Use random id so each invocation will set up a new cron schedule unless you override with a specific id.
    let workflow_id = format!("cron-{}", Uuid::new_v4());

    // This starts the cron workflow with a 5-second schedule.
    info!("Starting cron_counter_workflow as a cron schedule (every 5s)…");

    let input = vec![json!(0_u64).as_json_payload()?];

    let options = WorkflowOptions {
        cron_schedule: Some("*/5 * * * * *".into()), // every 5 seconds
        ..Default::default()
    };

    let start_res = client
        .start_workflow(
            input,
            "cron-workflows".to_string(),
            workflow_id.clone(),
            "cron_counter_workflow".to_string(),
            None,
            options,
        )
        .await?;

    println!(
        "Started cron workflow with id={workflow_id}, run_id={}",
        start_res.run_id
    );
    println!("It will run every 5 seconds until terminated.");
    Ok(())
}
