//! Workflow that blocks until it receives the `unblock` signal or cancellation.

use futures_util::StreamExt;
use log::info;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};

/// Signal name constant – kept in sync between client & workflow.
pub const UNBLOCK_SIGNAL: &str = "unblock";

/// Workflow that waits on a signal (or cancellation) before completing.
///
/// The workflow demonstrates [`WfContext::make_signal_channel`]. It listens on the
/// `"unblock"` signal channel. Whichever event arrives first – either the signal
/// or workflow cancellation – determines its result.
pub async fn signal_await_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // Create a drainable stream for the `unblock` signal.
    let mut sig_chan = ctx.make_signal_channel(UNBLOCK_SIGNAL);

    // Future that resolves if/when the workflow is cancelled.
    let cancel_fut = ctx.cancelled();

    info!(target: "signals_unblock", "Workflow started – waiting for signal or cancel");

    tokio::select! {
        _ = cancel_fut => {
            info!(target: "signals_unblock", "Workflow cancelled before signal arrived");
            Ok(WfExitValue::Cancelled)
        }
        _ = sig_chan.next() => {
            info!(target: "signals_unblock", "Received 'unblock' signal – completing");
            Ok(WfExitValue::Normal("unblocked".to_string()))
        }
    }
}
