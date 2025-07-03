//! Workflow: sleeps N seconds, sends email, loops, until `complete` signal arrives.
use futures_util::{FutureExt, StreamExt};
use log::info;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    AsJsonPayloadExt, FromJsonPayloadExt, workflow_commands::ActivityCancellationType,
};

const DEFAULT_INTERVAL_SECS: u64 = 2_592_000; // 30 days
const EMAIL_ACTIVITY_TYPE: &str = "send_email";
const COMPLETE_SIGNAL: &str = "complete";

/// Workflow args: Option<u64> interval_secs (default 30d)
pub async fn sleep_for_days_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // Accept single u64 arg for interval, else default
    let interval_secs = ctx
        .get_args()
        .first()
        .map(u64::from_json_payload)
        .transpose()? // Option<Result<_>> to Result<Option<_>>
        .unwrap_or(DEFAULT_INTERVAL_SECS);

    info!(target: "sleep_for_days", "Workflow started (interval_secs={interval_secs})");
    let mut complete_chan = ctx.make_signal_channel(COMPLETE_SIGNAL);
    let mut cycles = 0u32;
    loop {
        info!(target: "sleep_for_days", "Workflow sleeping for {interval_secs}s (cycle={cycles})");
        // Sleep via workflow timer
        ctx.timer(std::time::Duration::from_secs(interval_secs))
            .await;
        info!(target: "sleep_for_days", "Timer done; sending email activity");
        // Trigger email activity (can send a string with cycle count, e.g. subject)
        let subject = format!(
            "Periodic email after {} seconds (cycle {})",
            interval_secs,
            cycles + 1
        );
        ctx.activity(ActivityOptions {
            activity_type: EMAIL_ACTIVITY_TYPE.into(),
            input: subject.as_json_payload()?,
            start_to_close_timeout: Some(std::time::Duration::from_secs(20)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
        cycles += 1;

        // Check signal channel after each cycle using StreamExt::next()
        // (non-blocking)
        if let Some(_sig) = complete_chan.next().now_or_never().flatten() {
            info!(target: "sleep_for_days", "Received 'complete' signal – finishing workflow");
            break;
        }
    }
    Ok(WfExitValue::Normal("completed via signal".into()))
}
