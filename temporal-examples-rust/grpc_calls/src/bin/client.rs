//! Example: raw gRPC call to Temporal Frontend – ListNamespaces.

use env_logger::Env;
use log::info;

use tonic::transport::Channel;

// The generated gRPC client is re-exported from sdk-core protos.
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::ListNamespacesRequest;
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::workflow_service_client::WorkflowServiceClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise simple stdout logger.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Default Temporal dev-server frontend endpoint.
    let endpoint =
        std::env::var("TEMPORAL_ADDRESS").unwrap_or_else(|_| "http://localhost:7233".to_string());

    info!("Connecting to Temporal Frontend at {endpoint} …");

    // Build an async tonic channel.
    let channel = Channel::from_shared(endpoint.clone())?.connect().await?;

    // Construct the gRPC stub client.
    let mut client = WorkflowServiceClient::new(channel);

    // Perform the ListNamespaces RPC.
    let resp = client
        .list_namespaces(ListNamespacesRequest {
            page_size: 1000,
            ..Default::default()
        })
        .await?;

    println!("Namespaces returned by ListNamespaces:");
    for ns in resp.into_inner().namespaces {
        let info = ns.namespace_info.expect("namespace_info");
        println!("- {} (state: {:?})", info.name, info.state());
    }

    Ok(())
}
