//! Workflow demonstrating the Continue-As-New pattern.

use log::info;
use std::time::Duration;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    AsJsonPayloadExt, FromJsonPayloadExt, workflow_commands::ContinueAsNewWorkflowExecution,
};

/// Workflow that loops up to 10 times via Continue-As-New.
///
/// The workflow expects a single `u32` argument – the current iteration. On each
/// run it sleeps for one second and, if the iteration counter is below 10,
/// continues as new with the counter incremented. After reaching 10 the workflow
/// completes normally.
pub async fn looping_workflow(ctx: WfContext) -> WorkflowResult<()> {
    // Parse first argument (iteration) or default to 0 when no args present.
    let iteration: u32 = ctx
        .get_args()
        .first()
        .map(u32::from_json_payload)
        .transpose()? // Option<Result<_>> -> Result<Option<_>>
        .unwrap_or(0);

    info!(target: "continue_as_new", "Looping workflow iteration={iteration}");

    // Base case – after 10 iterations finish the workflow.
    if iteration >= 10 {
        return Ok(WfExitValue::Normal(()));
    }

    // Sleep one second to make progress visible.
    ctx.timer(Duration::from_secs(1)).await;

    // Build Continue-As-New command with incremented iteration.
    let can_cmd = ContinueAsNewWorkflowExecution {
        workflow_type: "looping_workflow".to_string(),
        task_queue: "continue-as-new".to_string(),
        arguments: vec![(iteration + 1).as_json_payload()?],
        ..Default::default()
    };

    Ok(WfExitValue::continue_as_new(can_cmd))
}
