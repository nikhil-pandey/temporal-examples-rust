//! Producer workflow: signals each message and then a 'done' signal to a consumer

use log::info;
use serde::Deserialize;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;

use crate::consumer_workflow::{DONE_SIGNAL, MSG_SIGNAL};

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct ProducerInput {
    pub consumer_id: String,
    pub messages: Vec<String>,
}

/// Sends each message as signal, then sends 'done', completes.
pub async fn producer_workflow(ctx: WfContext) -> WorkflowResult<()> {
    // One payload with ProducerInput JSON
    let arg_payload = ctx
        .get_args()
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing argument"))?;
    let input: ProducerInput = serde_json::from_slice(&arg_payload.data)?;

    info!(target: "message_passing", "Producer started for consumer_id={} ({} messages)", input.consumer_id, input.messages.len());

    // Signal each message to consumer
    for msg in &input.messages {
        let payload = msg.as_json_payload()?;
        ctx.signal_workflow(temporal_sdk::SignalWorkflowOptions::new(
            input.consumer_id.clone(),
            "",
            MSG_SIGNAL,
            vec![payload],
        ))
        .await
        .ok();
    }
    // Send done signal
    // For an empty signal payload we still need to provide an explicit vec with
    // concrete item type so the compiler can infer `impl Into<Payload>`.
    use temporal_sdk_core_protos::temporal::api::common::v1::Payload;

    ctx.signal_workflow(temporal_sdk::SignalWorkflowOptions::new(
        input.consumer_id.clone(),
        "",
        DONE_SIGNAL,
        Vec::<Payload>::new(),
    ))
    .await
    .ok();

    info!(target: "message_passing", "Producer finished sending");
    Ok(WfExitValue::Normal(()))
}
