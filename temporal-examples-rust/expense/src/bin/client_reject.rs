//! Client: send reject signal to running expense workflow
use env_logger::Env;
use expense::workflow::REJECT_SIGNAL;
use helpers::get_client;
use log::info;
use temporal_client::WorkflowClientTrait;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args: Vec<String> = std::env::args().collect();
    let workflow_id = args.get(1).expect("usage: reject <workflow_id>");
    let client = get_client().await?;
    info!("Sending reject signal to workflow_id={workflow_id}");
    client
        .signal_workflow_execution(
            workflow_id.to_string(),
            String::new(),
            REJECT_SIGNAL.to_string(),
            None,
            None,
        )
        .await?;
    println!("Sent 'reject' signal to {workflow_id}");
    Ok(())
}
