//! Workflow: logs before and after activity call and returns the activity result.
use log::info;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};

/// Workflow receives a String arg, logs it, calls activity, logs again, returns string.
pub async fn logging_workflow(ctx: WfContext) -> WorkflowResult<String> {
    let arg: String = ctx
        .get_args()
        .first()
        .map(String::from_json_payload)
        .transpose()? // Option<Result<T>> -> Result<Option<T>>
        .unwrap_or("default-msg".to_owned());

    info!(target: "custom_logger", "Workflow received message: {arg}");

    let act_handle = ctx
        .activity(ActivityOptions {
            activity_type: "log_activity".to_string(),
            input: arg.as_json_payload()?,
            start_to_close_timeout: Some(std::time::Duration::from_secs(10)),
            ..Default::default()
        })
        .await;

    let result: String = temporal_sdk_core_protos::coresdk::FromJsonPayloadExt::from_json_payload(
        act_handle
            .status
            .as_ref()
            .and_then(|s| match s {
                temporal_sdk_core_protos::coresdk::activity_result::activity_resolution::Status::Completed(c) => c.result.as_ref(),
                _ => None,
            })
            .expect("missing activity result payload"),
    )?;
    info!(target: "custom_logger", "Workflow got result from activity: {result}");

    Ok(WfExitValue::Normal(result))
}
