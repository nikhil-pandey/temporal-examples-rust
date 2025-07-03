//! Activity that heartbeats progress and supports cancellation.

use log::info;
use serde::{Deserialize, Serialize};
use temporal_sdk::{ActContext, ActivityError};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};

/// Payload for `fake_progress_activity` – currently only heartbeat sleep interval.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FakeProgressInput {
    pub sleep_interval_ms: u64,
}

/// Long-running activity that records its progress via heartbeats every
/// `sleep_interval_ms` milliseconds. If the associated workflow is cancelled
/// the activity observes that via [`ActContext::is_cancelled`] and returns the
/// last progress value instead of running to completion.
pub async fn fake_progress_activity(
    ctx: ActContext,
    input: FakeProgressInput,
) -> Result<u64, ActivityError> {
    info!("Starting fake progress activity");

    // Resume from last recorded heartbeat if present.
    let starting_point = match ctx.get_heartbeat_details().first() {
        Some(hb) => u64::from_json_payload(hb)?,
        None => 1,
    };

    let sleep_ms = input.sleep_interval_ms;

    // Ping loop runs until 100 or cancellation.
    let mut count = starting_point;
    while count <= 100 {
        println!("Progress: {count}");

        // Record heartbeat with latest progress.
        ctx.record_heartbeat(vec![
            count
                .as_json_payload()
                .expect("Could not serialize heartbeat"),
        ]);

        // Respect cancellation – break out early if workflow requested it.
        if ctx.is_cancelled() {
            println!("### Activity canceled ###");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
        count += 1;
    }

    Ok(count)
}
