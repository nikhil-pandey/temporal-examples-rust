//! Workflow that runs order processing concurrently with a timer. If the
//! processing exceeds the caller-provided threshold we send an email.

use helpers::parse_activity_result;
use log::{debug, info};
use std::time::Duration;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    workflow_commands::ActivityCancellationType, AsJsonPayloadExt, FromJsonPayloadExt,
};

const ORDER_PROCESSING_ACTIVITY: &str = "order_processing_activity";
const SEND_EMAIL_ACTIVITY: &str = "send_email_activity";

pub async fn sample_timer_workflow(ctx: WfContext) -> WorkflowResult<()> {
    // Extract optional threshold argument.
    let threshold: u64 = ctx
        .get_args()
        .first()
        .map(u64::from_json_payload)
        .transpose()? // Option<Result<_>> -> Result<Option<_>>
        .unwrap_or(2);

    debug!("Timer workflow threshold={threshold}s");

    // Start order processing activity.
    let order_fut = ctx.activity(ActivityOptions {
        activity_type: ORDER_PROCESSING_ACTIVITY.to_string(),
        input: "".as_json_payload()?,
        start_to_close_timeout: Some(Duration::from_secs(10)),
        cancellation_type: ActivityCancellationType::Abandon,
        ..Default::default()
    });

    // Start timer.
    let timer_fut = ctx.timer(Duration::from_secs(threshold));

    // Run concurrently; we don't need timer result beyond its completion.
    let (order_res, _) = tokio::join!(order_fut, timer_fut);

    // Determine if order finished.
    let order_completed_ok = parse_activity_result::<()>(&order_res).is_ok();

    if !order_completed_ok {
        info!("Processing exceeded threshold — triggering email notification");
        ctx.activity(ActivityOptions {
            activity_type: SEND_EMAIL_ACTIVITY.to_string(),
            input: "".as_json_payload()?,
            start_to_close_timeout: Some(Duration::from_secs(10)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
    }

    Ok(WfExitValue::Normal(()))
}
