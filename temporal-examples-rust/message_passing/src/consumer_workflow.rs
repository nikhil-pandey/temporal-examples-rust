//! Consumer workflow: receives signals with messages, accumulates, exits on "done" signal.

//! Consumer workflow: receives signals with messages, accumulates, exits on "done" signal.

use futures_util::StreamExt;
use log::info;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::FromJsonPayloadExt;

pub const MSG_SIGNAL: &str = "add_message";
pub const DONE_SIGNAL: &str = "done";

#[derive(Default)]
struct ConsumerState {
    messages: Vec<String>,
    done: bool,
}

/// Receives 'add_message' signals (String payloads) and a 'done' signal, then returns Vec<String>
pub async fn consumer_workflow(ctx: WfContext) -> WorkflowResult<Vec<String>> {
    let mut state = ConsumerState::default();

    let mut msg_channel = ctx.make_signal_channel(MSG_SIGNAL);
    let mut done_channel = ctx.make_signal_channel(DONE_SIGNAL);

    info!(target: "message_passing", "Consumer workflow started");

    while !state.done {
        tokio::select! {
            Some(sig) = msg_channel.next() => {
                for p in sig.input {
                    match String::from_json_payload(&p) {
                        Ok(msg) => {
                            info!(target: "message_passing", "Received message: {msg}");
                            state.messages.push(msg);
                        },
                        Err(e) => info!(target: "message_passing", "Failed to decode message: {e}"),
                    }
                }
            }
            Some(_sig) = done_channel.next() => {
                info!(target: "message_passing", "Received done signal");
                state.done = true;
            }
            _ = ctx.cancelled() => {
                info!(target: "message_passing", "Consumer cancelled");
                return Ok(WfExitValue::Cancelled);
            }
        }
    }
    info!(target: "message_passing", "Consumer finishing with {} messages", state.messages.len());
    Ok(WfExitValue::Normal(state.messages))
}
