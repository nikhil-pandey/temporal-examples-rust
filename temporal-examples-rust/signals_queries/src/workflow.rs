//! Workflow that exposes a query and signal to unblock/wait, and handles cancellation.

use futures_util::StreamExt;
use log::{info, warn};
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};

pub const BLOCKED_QUERY: &str = "blocked";
pub const UNBLOCK_SIGNAL: &str = "unblock";
pub const TASK_QUEUE: &str = "signals-queries";

/// Workflow with a bool state "blocked"; can be queried, signalled, or cancelled.
pub async fn signals_queries_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // The workflow tracks blocked/unblocked state.
    let mut blocked = true;
    // [NOTE] Query registration API is not available in Rust SDK as of this example.
    // To add query support, see the README for SDK updates.

    let mut sig_chan = ctx.make_signal_channel(UNBLOCK_SIGNAL);

    info!(target: "signals_queries", "Workflow started, initially blocked=true.");

    loop {
        tokio::select! {
            // Signal received – change state to unblocked
            Some(_sig) = sig_chan.next() => {
                blocked = false;
                info!(target: "signals_queries", "Received '{UNBLOCK_SIGNAL}' signal – unblocking workflow");
            }
            // Cancellation from client
            _ = ctx.cancelled() => {
                warn!(target: "signals_queries", "Workflow cancelled.");
                return Ok(WfExitValue::Normal("cancelled".to_string()));
            }
            // Small sleep to yield when still blocked to prevent busy loop
            _ = ctx.timer(std::time::Duration::from_millis(100)), if blocked => { /* continue looping */ }
        }

        // If state changed to unblocked, finish.
        if !blocked {
            info!(target: "signals_queries", "Workflow exiting normally after unblock");
            return Ok(WfExitValue::Normal("unblocked".to_string()));
        }
    }
}
