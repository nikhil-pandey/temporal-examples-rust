//! Workflow that invokes both activities with DI'd services.
use log::info;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    workflow_commands::ActivityCancellationType, AsJsonPayloadExt, FromJsonPayloadExt,
};

/// Workflow that calls `greet_english` and `greet_spanish` activities.
pub async fn multi_greet_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // Get name arg or default
    let name: String = ctx
        .get_args()
        .first()
        .map(String::from_json_payload)
        .transpose()? // Option<Result<T>> -> Result<Option<T>>
        .unwrap_or("World".to_string());

    // Each activity: pass name as single argument (as payload)
    let en = ctx
        .activity(ActivityOptions {
            activity_type: "greet_english".into(),
            input: name.as_json_payload()?,
            start_to_close_timeout: Some(std::time::Duration::from_secs(10)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
    let en_greeting: String =
        temporal_sdk_core_protos::coresdk::FromJsonPayloadExt::from_json_payload(
            en.status
                .as_ref()
                .and_then(|status| match status {
                    temporal_sdk_core_protos::coresdk::activity_result::activity_resolution::Status::Completed(comp) => comp.result.as_ref(),
                    _ => None,
                })
                .expect("greet_english failed"),
        )?;

    let es = ctx
        .activity(ActivityOptions {
            activity_type: "greet_spanish".into(),
            input: name.as_json_payload()?,
            start_to_close_timeout: Some(std::time::Duration::from_secs(10)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
    let es_greeting: String =
        temporal_sdk_core_protos::coresdk::FromJsonPayloadExt::from_json_payload(
            es.status
                .as_ref()
                .and_then(|status| match status {
                    temporal_sdk_core_protos::coresdk::activity_result::activity_resolution::Status::Completed(comp) => comp.result.as_ref(),
                    _ => None,
                })
                .expect("greet_spanish failed"),
        )?;

    info!("Workflow: gathered two greetings");
    let output = format!("{en_greeting}\n{es_greeting}");
    Ok(WfExitValue::Normal(output))
}
