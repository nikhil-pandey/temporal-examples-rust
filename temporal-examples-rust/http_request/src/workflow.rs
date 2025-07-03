//! Workflow definition for the HTTP request example.

use helpers::parse_activity_result;
use log::{info, warn};
use std::time::Duration;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    AsJsonPayloadExt, workflow_commands::ActivityCancellationType,
};

/// Workflow that calls the `make_http_request` activity and returns the echoed
/// `answer` string.
pub async fn http_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // Kick off the activity immediately – we don't need to pass any input.
    let act_handle = ctx
        .activity(ActivityOptions {
            activity_type: "make_http_request".to_string(),
            input: "".as_json_payload()?,
            // Reasonable timeouts. These can be tuned as needed.
            start_to_close_timeout: Some(Duration::from_secs(30)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;

    match parse_activity_result::<String>(&act_handle) {
        Ok(answer) => {
            info!("Activity completed with answer={answer}");
            Ok(WfExitValue::Normal(answer))
        }
        Err(err) => {
            warn!("Activity failed: {err}");
            Ok(WfExitValue::Evicted)
        }
    }
}
