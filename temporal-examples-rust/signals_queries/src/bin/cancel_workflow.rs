//! Client binary to cancel a workflow.
use env_logger::Env;
use helpers::get_client;
use log::info;

// No-op: TASK_QUEUE import not used.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let workflow_id = std::env::args().nth(1).expect("Require workflow id as arg");
    let client = get_client().await?;
    info!(target: "signals_queries", "Cancelling workflow id={workflow_id}");
    client
        .cancel_workflow_execution(workflow_id.clone(), None, "cancel-cli".to_string(), None)
        .await?;
    println!("Cancelled workflow {workflow_id}");
    Ok(())
}
use temporal_client::WorkflowClientTrait;
