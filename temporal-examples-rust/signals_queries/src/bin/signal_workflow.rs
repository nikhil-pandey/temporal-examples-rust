//! Client binary to send 'unblock' signal to workflow.
use env_logger::Env;
use helpers::get_client;
use log::info;
use signals_queries::workflow::UNBLOCK_SIGNAL;
use temporal_client::WorkflowClientTrait;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let workflow_id = std::env::args().nth(1).expect("Require workflow id as arg");
    let client = get_client().await?;

    info!(target: "signals_queries", "Sending unblock signal to workflow id={workflow_id}");

    client
        .signal_workflow_execution(
            workflow_id.clone(),
            String::new(),
            UNBLOCK_SIGNAL.to_string(),
            None,
            None,
        )
        .await?;

    println!("Sent 'unblock' signal to workflow {workflow_id}");
    Ok(())
}
