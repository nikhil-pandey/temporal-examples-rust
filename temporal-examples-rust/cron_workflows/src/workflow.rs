//! Cron workflow that logs and returns current run count for each scheduled invocation.

use log::info;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
// Only need the helper trait to deserialize the optional previous counter.
use temporal_sdk_core_protos::coresdk::FromJsonPayloadExt;

/// The cron workflow. Each run receives the previous counter and increments it.
///   - On the first run, no input is provided, so we start at 1.
///   - On each subsequent invocation, we expect a single integer argument: the run count.
pub async fn cron_counter_workflow(ctx: WfContext) -> WorkflowResult<u64> {
    // Get the run count from previous input, or start from 1.
    let prev_count: u64 = ctx
        .get_args()
        .first()
        .map(u64::from_json_payload)
        .transpose()? // Option<Result<T>> -> Result<Option<T>>
        .unwrap_or(1);

    // Temporal server increments attempt for each scheduled run! This is also available via ctx.get_info().attempt, but we use state for demonstration.
    let count = prev_count + 1;

    info!(target: "cron_workflows", "Cron workflow run: count={count} (prev={prev_count})");

    // Complete the workflow with the new count as output (can be used as input to next run)
    Ok(WfExitValue::Normal(count))
}
