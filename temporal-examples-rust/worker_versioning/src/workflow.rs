//! Workflow for Build-ID based worker versioning demo.

use log::info;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::FromJsonPayloadExt;

/// Demo workflow – returns a greeting with current build id embedded.
/// The input is expected to be Some(String) build id; if not, output says unknown.
pub async fn versioned_greeting_workflow(ctx: WfContext) -> WorkflowResult<String> {
    let build_id = ctx
        .get_args()
        .first()
        .map(String::from_json_payload)
        .transpose()? // Option<Result<T>> -> Result<Option<T>>
        .unwrap_or_else(|| "unknown".to_string());
    let greeting = format!("Hello from worker Build ID: {build_id}!");
    info!(target: "worker_versioning", "Workflow returning: {greeting}");
    Ok(WfExitValue::Normal(greeting))
}
