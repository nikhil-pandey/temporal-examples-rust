//! Parent workflow that spawns two child workflows concurrently.

use log::info;
// Use high-level re-exports directly rather than the nested module path now
// exposed by `temporal_sdk`.
use temporal_sdk::{ChildWorkflowOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;

/// Parent workflow – spawns two children and returns their combined results.
pub async fn parent_workflow(ctx: WfContext) -> WorkflowResult<Vec<String>> {
    info!("Parent workflow started – spawning children");

    // Helper to start a child with given number
    let start_child = |num: u32| {
        let opts = ChildWorkflowOptions {
            workflow_id: format!("child-{num}"),
            workflow_type: "child_workflow".to_string(),
            task_queue: Some("child-workflows".to_string()),
            input: vec![num.as_json_payload().expect("payload")],
            ..Default::default()
        };
        ctx.child_workflow(opts).start(&ctx)
    };

    // Kick off both child workflows in parallel.
    let child1_fut = start_child(1);
    let child2_fut = start_child(2);

    // Wait for pending to start to convert into started handles.
    let pending1 = child1_fut.await;
    let pending2 = child2_fut.await;

    let started1 = pending1
        .into_started()
        .ok_or_else(|| anyhow::anyhow!("child 1 failed to start"))?;
    let started2 = pending2
        .into_started()
        .ok_or_else(|| anyhow::anyhow!("child 2 failed to start"))?;

    // Wait for completion concurrently.
    // Wait for both children to complete concurrently.
    let (res1, res2) = tokio::join!(started1.result(), started2.result());

    // The `result()` future resolves to a `ChildWorkflowResult`, so we can use the
    // values directly without `?` operators.
    let payload1 = res1;
    let payload2 = res2;

    // Each ChildWorkflowResult deserializes automatically? Actually result() returns ChildWorkflowResult which contains payloads etc.
    // Use helpers: The ChildWorkflowResult implements FromJsonPayload maybe but easiest: use AsJsonPayloadExt on vector of Payload.
    use temporal_sdk_core_protos::coresdk::child_workflow::child_workflow_result::Status;
    use temporal_sdk_core_protos::coresdk::child_workflow::ChildWorkflowResult;
    use temporal_sdk_core_protos::coresdk::FromJsonPayloadExt;

    fn extract(r: ChildWorkflowResult) -> anyhow::Result<String> {
        match r.status.ok_or_else(|| anyhow::anyhow!("missing status"))? {
            Status::Completed(c) => {
                let p = c.result.ok_or_else(|| anyhow::anyhow!("missing payload"))?;
                Ok(String::from_json_payload(&p)?)
            }
            other => Err(anyhow::anyhow!("child failed: {:?}", other)),
        }
    }

    let out1 = extract(payload1);
    let out2 = extract(payload2);

    let results = vec![out1?, out2?];

    Ok(WfExitValue::Normal(results))
}
