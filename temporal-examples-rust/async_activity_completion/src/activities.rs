//! Activity implementations for the async activity completion example.

use log::info;
use std::{sync::Arc, time::Duration};
use temporal_sdk::{ActContext, ActExitValue, ActivityError};
use tokio::sync::Mutex;

use helpers::get_client;
use temporal_client::WorkflowClientTrait;

/// Activity that demonstrates asynchronous completion using
/// [`AsyncCompletionClient`].
///
/// The activity immediately returns [`ActExitValue::WillCompleteAsync`]. In the
/// background it spawns a Tokio task that waits 5 seconds and then completes
/// itself via the Temporal client.
pub async fn do_something_async(
    ctx: ActContext,
    _payload: Option<String>,
) -> Result<ActExitValue<()>, ActivityError> {
    // Store the task token in an `Arc<Mutex<_>>` so the spawned task can take
    // ownership of it.
    let task_token: Arc<Mutex<Option<Vec<u8>>>> =
        Arc::new(Mutex::new(Some(ctx.get_info().task_token.clone())));

    let token_ref = Arc::clone(&task_token);

    // Spawn detached task that will complete the activity asynchronously.
    tokio::spawn(async move {
        // Pretend to do some expensive work.
        info!("doing work in background …");
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Acquire the token and complete the activity.
        if let Some(token) = token_ref.lock().await.take() {
            if let Err(err) = complete_activity(token).await {
                // Log the error – in production you might retry or send to
                // Sentry etc.
                log::error!("failed to complete activity: {err}");
            }
        }
    });

    // Indicate to the Temporal server that the activity will complete later.
    Ok(ActExitValue::WillCompleteAsync)
}

async fn complete_activity(token: Vec<u8>) -> anyhow::Result<()> {
    use temporal_sdk_core::TaskToken;
    use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;

    let client = get_client().await?;

    client
        .complete_activity_task(TaskToken(token), Some("done".as_json_payload()?.into()))
        .await?;

    info!("activity marked complete");
    Ok(())
}
