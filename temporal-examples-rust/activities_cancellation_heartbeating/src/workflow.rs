//! Workflow demonstrating activity cancellation & heartbeating.

use helpers::parse_activity_result;
use log::{info, warn};
use prost_wkt_types::Duration as ProstDuration;
use std::time::Duration;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core::protos::temporal::api::common::v1::RetryPolicy;
use temporal_sdk_core_protos::coresdk::{
    workflow_commands::ActivityCancellationType, AsJsonPayloadExt,
};

use crate::activities::FakeProgressInput;

/// Main workflow – runs a long running activity with heartbeats and shows how
/// to cancel it.
pub async fn run_cancellable_activity(ctx: WfContext) -> WorkflowResult<u64> {
    info!("Inside run_cancellable_activity");

    // Start long-running heartbeat activity.
    let fake_progress_handle = ctx.activity(ActivityOptions {
        activity_type: "fake_progress_activity".to_string(),
        cancellation_type: ActivityCancellationType::WaitCancellationCompleted,
        input: FakeProgressInput {
            sleep_interval_ms: 500,
        }
        .as_json_payload()?,
        heartbeat_timeout: Some(Duration::from_secs(3)),
        start_to_close_timeout: Some(Duration::from_secs(120)),
        retry_policy: Some(RetryPolicy {
            initial_interval: Some(ProstDuration {
                seconds: 1,
                nanos: 0,
            }),
            maximum_attempts: 0, // unlimited
            ..Default::default()
        }),
        ..Default::default()
    });

    // Start a second activity that will likely be skipped (for demo purposes).
    let skipped_handle = ctx.activity(ActivityOptions {
        activity_type: "skipped_activity".to_string(),
        cancellation_type: ActivityCancellationType::TryCancel,
        input: "".as_json_payload()?,
        heartbeat_timeout: Some(Duration::from_secs(3)),
        start_to_close_timeout: Some(Duration::from_secs(120)),
        ..Default::default()
    });

    // Observe workflow cancellation signal.
    let cancel_handle = ctx.cancelled();

    let mut final_value = 0u64;

    tokio::select! {
        _ = cancel_handle => {
            warn!("## workflow canceled ##");
        },
        res = async {
            let progress_result = fake_progress_handle.await;
            // skipped activity may still be pending – wait for it anyway.
            skipped_handle.await;
            progress_result
        } => {
            final_value = parse_activity_result(&res)?;
            info!("Activity completed, value={final_value}");
        }
    }

    // Always run cleanup activity.
    ctx.activity(ActivityOptions {
        activity_type: "cleanup_activity".to_string(),
        cancellation_type: ActivityCancellationType::TryCancel,
        input: "".as_json_payload()?,
        heartbeat_timeout: Some(Duration::from_secs(3)),
        start_to_close_timeout: Some(Duration::from_secs(120)),
        ..Default::default()
    })
    .await;

    Ok(WfExitValue::Normal(final_value))
}
