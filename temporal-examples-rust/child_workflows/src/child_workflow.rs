//! A trivial child workflow implementation.

use log::info;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::FromJsonPayloadExt;

/// Child workflow that sleeps one second and returns a message.
///
/// It expects a single `u32` argument indicating the child number.
pub async fn child_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // Extract the single `u32` argument passed to the workflow.
    let arg_payload = ctx
        .get_args()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing arg"))?;

    let num: u32 = u32::from_json_payload(arg_payload)?;

    info!("Child workflow {num} started");

    // Sleep 1 second using workflow timer.
    ctx.timer(std::time::Duration::from_secs(1)).await;

    let res = format!("child {num} done");
    Ok(WfExitValue::Normal(res))
}
