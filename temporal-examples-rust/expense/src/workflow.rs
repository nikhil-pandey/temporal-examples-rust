//! Expense approval workflow, mirrors TypeScript sample.
use crate::activities::Expense;
use futures_util::StreamExt;
use helpers::parse_activity_result;
use log::{info, warn};
use serde_json::json;
use std::time::Duration;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    workflow_commands::ActivityCancellationType, AsJsonPayloadExt,
};

// Signal names
pub const APPROVE_SIGNAL: &str = "approve";
pub const REJECT_SIGNAL: &str = "reject";

/// Workflow: Creates expense, waits for approval, timeout, or rejection, marks result.
pub async fn expense_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // Parse args: amount, requester, description (JSON, for demo)
    let args: serde_json::Value = ctx
        .get_args()
        .first()
        .map(|p| serde_json::from_slice(&p.data).unwrap_or_else(|_| json!({})))
        .unwrap_or_else(
            || json!({"amount": 42.5, "requester": "unknown", "description": "demo expense"}),
        );

    // Call create_expense activity (returns Expense object)
    let expense_future = ctx.activity(ActivityOptions {
        activity_type: "create_expense".to_string(),
        input: args.to_string().as_json_payload()?,
        start_to_close_timeout: Some(Duration::from_secs(20)),
        cancellation_type: ActivityCancellationType::TryCancel,
        ..Default::default()
    });
    let mut expense: Expense = parse_activity_result(&expense_future.await)?;

    // Open approval signals
    let mut approve_chan = ctx.make_signal_channel(APPROVE_SIGNAL);
    let mut reject_chan = ctx.make_signal_channel(REJECT_SIGNAL);
    let timeout = ctx.timer(Duration::from_secs(30));
    // let mut status = None;
    info!("Waiting for approve/reject signal or timeout...");
    tokio::select! {
        _ = approve_chan.next() => {
            info!("Received approval signal");
            expense.status = "approved".into();
            // status = Some("approved");
        }
        _ = reject_chan.next() => {
            info!("Received rejection signal");
            expense.status = "rejected".into();
            // status = Some("rejected");
        }
        _ = timeout => {
            warn!("Timeout expired - expense auto-timed out");
            expense.status = "timed_out".into();
            // status = Some("timed_out");
        }
    }

    // Call http_post (could fail/ignored for log)
    let _ = ctx
        .activity(ActivityOptions {
            activity_type: "http_post".into(),
            input: serde_json::to_string(&expense)?.as_json_payload()?,
            start_to_close_timeout: Some(Duration::from_secs(10)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;

    // Call complete_expense to change status finalized
    let final_expense: Expense = parse_activity_result(
        &ctx.activity(ActivityOptions {
            activity_type: "complete_expense".into(),
            input: serde_json::to_string(&expense)?.as_json_payload()?,
            start_to_close_timeout: Some(Duration::from_secs(10)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await,
    )?;

    Ok(WfExitValue::Normal(format!(
        "Expense {} is {} (desc: {})",
        final_expense.request_id, final_expense.status, final_expense.description
    )))
}
