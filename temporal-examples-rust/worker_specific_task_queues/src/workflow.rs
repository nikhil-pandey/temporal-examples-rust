//! Workflow: Calls greet activity on queue-a and queue-b with task_queue overriden.
use helpers::parse_activity_result;
use log::info;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    workflow_commands::ActivityCancellationType, AsJsonPayloadExt,
};

/// Workflow: call greet_from_queue activity on both task queues, return Vec<String>.
pub async fn multi_queue_workflow(ctx: WfContext) -> WorkflowResult<Vec<String>> {
    // Call greet on queue-a
    let greeting_a = ctx
        .activity(ActivityOptions {
            activity_type: "greet_from_queue".to_string(),
            input: "queue-a".as_json_payload()?,
            start_to_close_timeout: Some(std::time::Duration::from_secs(10)),
            task_queue: Some("queue-a".to_string()),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
    let result_a: String = parse_activity_result(&greeting_a)?;

    // Call greet on queue-b
    let greeting_b = ctx
        .activity(ActivityOptions {
            activity_type: "greet_from_queue".to_string(),
            input: "queue-b".as_json_payload()?,
            start_to_close_timeout: Some(std::time::Duration::from_secs(10)),
            task_queue: Some("queue-b".to_string()),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
    let result_b: String = parse_activity_result(&greeting_b)?;

    info!(target: "worker_specific_task_queues", "Workflow returning: [a={result_a}, b={result_b}]");
    Ok(WfExitValue::Normal(vec![result_a, result_b]))
}
