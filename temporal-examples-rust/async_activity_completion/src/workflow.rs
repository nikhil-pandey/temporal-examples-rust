//! Workflow definition for the async activity completion example.

use helpers::parse_activity_result;
use log::{info, warn};
use std::time::Duration;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    AsJsonPayloadExt, workflow_commands::ActivityCancellationType,
};

/// Workflow that invokes `do_something_async` and waits for the result the
/// background task will eventually produce.
pub async fn async_activity_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // Kick off the activity right away. We don't need to pass any real input.
    let act_handle = ctx
        .activity(ActivityOptions {
            activity_type: "do_something_async".to_string(),
            input: "".as_json_payload()?,
            start_to_close_timeout: Some(Duration::from_secs(30)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;

    match parse_activity_result::<String>(&act_handle) {
        Ok(value) => {
            info!("activity completed with value={value}");
            Ok(WfExitValue::Normal(value))
        }
        Err(err) => {
            warn!("activity failed: {err}");
            Ok(WfExitValue::Evicted)
        }
    }
}
