//! Activity that simulates a long-running order processing operation.

use log::info;
use rand::Rng;
use temporal_sdk::{ActContext, ActivityError};

/// Randomly sleeps between 1-10 seconds to emulate work.
pub async fn order_processing_activity(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<(), ActivityError> {
    // Pick random sleep duration.
    let time_needed = rand::thread_rng().gen_range(1..=10);

    // Inline formatting style per clippy.
    info!("Processing will take {time_needed}s");

    tokio::time::sleep(tokio::time::Duration::from_secs(time_needed)).await;

    Ok(())
}
