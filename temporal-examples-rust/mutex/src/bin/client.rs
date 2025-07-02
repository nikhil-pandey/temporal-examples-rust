//! Client: starts two test_lock_workflow executions to demonstrate serialization.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::{WfClientExt, WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = get_client().await?;

    let args: Vec<String> = std::env::args().collect();
    let lock_id = args.get(1).map(|s| s.as_str()).unwrap_or("mutex-lock");

    info!(target: "mutex", "Client: will start test_lock_workflow for lock_id={lock_id}");

    let client_a = client.clone();
    let client_b = client;
    let lock_id_a = lock_id.to_string();
    let lock_id_b = lock_id.to_string();

    // Move all values into tasks to fix borrow checker issues
    let f1 = tokio::spawn(async move {
        let input = vec![json!(lock_id_a).as_json_payload()?];
        let workflow_id = format!("test-mutex-A-{}", Uuid::new_v4());
        let start_res = client_a
            .start_workflow(
                input,
                "mutex".to_string(),
                workflow_id.clone(),
                "test_lock_workflow".to_string(),
                None,
                WorkflowOptions::default(),
            )
            .await?;
        info!(target: "mutex", "Started instance A: wf_id={workflow_id} run_id={}", start_res.run_id);
        let handle =
            client_a.get_untyped_workflow_handle(workflow_id.clone(), start_res.run_id.clone());
        let _res_payloads: Vec<Payload> = handle
            .get_workflow_result(Default::default())
            .await?
            .unwrap_success();
        Ok::<(), anyhow::Error>(())
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let f2 = tokio::spawn(async move {
        let input = vec![json!(lock_id_b).as_json_payload()?];
        let workflow_id = format!("test-mutex-B-{}", Uuid::new_v4());
        let start_res = client_b
            .start_workflow(
                input,
                "mutex".to_string(),
                workflow_id.clone(),
                "test_lock_workflow".to_string(),
                None,
                WorkflowOptions::default(),
            )
            .await?;
        info!(target: "mutex", "Started instance B: wf_id={workflow_id} run_id={}", start_res.run_id);
        let handle =
            client_b.get_untyped_workflow_handle(workflow_id.clone(), start_res.run_id.clone());
        let _res_payloads: Vec<Payload> = handle
            .get_workflow_result(Default::default())
            .await?
            .unwrap_success();
        Ok::<(), anyhow::Error>(())
    });
    let _ = futures_util::try_join!(f1, f2);

    println!("Both test workflow runs completed. Check logs for ordering!");

    Ok(())
}
