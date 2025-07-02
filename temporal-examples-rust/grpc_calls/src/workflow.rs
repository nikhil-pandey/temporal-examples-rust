//! A minimal "Hello World" workflow definition.

use log::info;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::FromJsonPayloadExt;

/// Workflow that greets the provided name.
///
/// The workflow expects a single string argument (the name). It returns a
/// greeting string.
pub async fn greet(ctx: WfContext) -> WorkflowResult<String> {
    // Extract first argument as a string. `get_args()` returns a slice of
    // payloads; we only care about the first one.
    // Grab the first argument passed into the workflow. Using `first()`
    // avoids Clippy's `get_first` lint while remaining clear and concise.
    let name_payload = ctx
        .get_args()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing workflow argument"))?;

    let name: String = String::from_json_payload(name_payload)?;

    let greeting = format!("Hello, {name}!");
    info!("workflow says: {greeting}");

    Ok(WfExitValue::Normal(greeting))
}
