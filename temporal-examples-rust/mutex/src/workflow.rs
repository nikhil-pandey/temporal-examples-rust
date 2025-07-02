//! Implements lock_workflow and test_lock_workflow for mutex example.

use futures_util::StreamExt;
use log::{info, warn};
use std::collections::VecDeque;
use temporal_sdk::SignalWorkflowOptions;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;

// Signal names
const LOCK_REQUESTED_SIGNAL: &str = "lock-requested";
const LOCK_ACQUIRED_SIGNAL: &str = "lock-acquired";
const LOCK_RELEASED_SIGNAL: &str = "lock-released";

/// The mutex/lock coordination workflow.
/// Maintains an internal FIFO queue of workflow IDs. Only the front is the holder.
pub async fn lock_workflow(ctx: WfContext) -> WorkflowResult<()> {
    // We'll queue workflow_id strings as requestors.
    let mut queue: VecDeque<String> = VecDeque::new();
    // Create a channel for incoming requests.
    let mut req_chan = ctx.make_signal_channel(LOCK_REQUESTED_SIGNAL);

    info!(target: "mutex", "lock_workflow started");
    loop {
        // Grant lock if queue nonempty and no one currently holds.
        while let Some(next_wfid) = queue.front() {
            let holder = next_wfid.clone();
            // Payload for grant/release signals: workflow id as payload
            let holder_id_payload = holder.as_json_payload().ok();
            let signal_payloads = if let Some(p) = &holder_id_payload {
                vec![p.clone()]
            } else {
                vec![]
            };
            // Use signal_workflow API
            info!(target: "mutex", "Granting lock to {holder}");
            let _ = ctx
                .signal_workflow(SignalWorkflowOptions::new(
                    holder.clone(),
                    "",
                    LOCK_ACQUIRED_SIGNAL,
                    signal_payloads.clone(),
                ))
                .await;
            // Now wait for release from this holder
            let mut release_chan = ctx.make_signal_channel(LOCK_RELEASED_SIGNAL);
            // Wait for release/exit/cancel
            loop {
                tokio::select! {
                    signal = release_chan.next() => {
                        if let Some(sig) = signal {
                            // Match signal by decoding first payload as the workflow id
                            if let Some(payload) = sig.input.first() {
                                if let Ok(releaser) = String::from_json_payload(payload) {
                                    if releaser == holder {
                                        info!(target: "mutex", "Received lock release from holder {releaser}");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    _ = ctx.cancelled() => {
                        warn!(target: "mutex", "Workflow cancelled");
                        return Ok(WfExitValue::Cancelled);
                    }
                }
            }
            // Remove the granted from the queue
            queue.pop_front();
        }
        // Wait for the next request
        tokio::select! {
            maybe_req = req_chan.next() => {
                if let Some(sig) = maybe_req {
                    // Signal input: workflow id will be in first payload of input
                    if let Some(payload) = sig.input.first() {
                        if let Ok(requestor) = String::from_json_payload(payload) {
                            info!(target: "mutex", "Received lock request from {requestor}");
                            if !queue.contains(&requestor) {
                                queue.push_back(requestor);
                            }
                        }
                    }
                }
            }
            _ = ctx.cancelled() => {
                warn!(target: "mutex", "Workflow cancelled");
                return Ok(WfExitValue::Cancelled);
            }
        }
    }
}

/// Test workflow: request the lock, wait for acquisition, do work, then release.
pub async fn test_lock_workflow(ctx: WfContext) -> WorkflowResult<()> {
    // Workflow expects as argument: lock_id (string)
    let lock_id: String = ctx
        .get_args()
        .first()
        .map(String::from_json_payload)
        .transpose()? // Option<Result<T>> -> Result<Option<T>>
        .unwrap_or("mutex-lock".to_owned());
    // Get our own workflow id
    // Use the lock test workflow's own id as its identity
    // (should match what client used as workflow id for itself)
    let my_id = ctx
        .get_args()
        .first()
        .map(String::from_json_payload)
        .transpose()? // Option<Result<T>> -> Result<Option<T>>
        .unwrap_or_else(|| "test-locker-unknown".to_string());
    // Prepare the payload that identifies us (the requesting workflow)
    let payloads: Vec<Payload> = vec![my_id.as_json_payload()?];
    info!(target: "mutex", "{my_id}: Requesting lock {lock_id}");
    let _ = ctx
        .signal_workflow(SignalWorkflowOptions::new(
            lock_id.clone(),
            "",
            LOCK_REQUESTED_SIGNAL,
            payloads.clone(),
        ))
        .await;
    // Wait for lock-acquired signal intended for us
    let mut acquired_chan = ctx.make_signal_channel(LOCK_ACQUIRED_SIGNAL);
    loop {
        tokio::select! {
            Some(sig) = acquired_chan.next() => {
                // Only acquire if signal intended for us
                if let Some(payload) = sig.input.first() {
                    if let Ok(target) = String::from_json_payload(payload) {
                        if target == my_id {
                            info!(target: "mutex", "{my_id}: Acquired lock {lock_id}");
                            break;
                        }
                    }
                }
            }
            _ = ctx.cancelled() => {
                warn!(target: "mutex", "test_lock_workflow cancelled");
                return Ok(WfExitValue::Cancelled);
            }
        }
    }
    // Do work for 5 seconds
    info!(target: "mutex", "{my_id}: Holding lock {lock_id} for 5s");
    ctx.timer(std::time::Duration::from_secs(5)).await;
    // Release: send lock-released signal with our workflow id as payload
    let release_payloads = vec![my_id.as_json_payload()?];
    let _ = ctx
        .signal_workflow(SignalWorkflowOptions::new(
            lock_id.clone(),
            "",
            LOCK_RELEASED_SIGNAL,
            release_payloads.clone(),
        ))
        .await;
    info!(target: "mutex", "{my_id}: Released lock {lock_id}");
    Ok(WfExitValue::Normal(()))
}
