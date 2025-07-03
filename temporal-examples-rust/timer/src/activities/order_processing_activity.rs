//! Activity that simulates a long-running order processing operation.

#![allow(deprecated)] // silence `rand::Rng::gen_range` deprecation warnings

use log::info;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use temporal_sdk::{ActContext, ActivityError};

/// Randomly sleeps between 1-10 seconds to emulate work.
pub async fn order_processing_activity(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<(), ActivityError> {
    // Pick a random sleep duration between 1-10 seconds.
    // Using `gen_range` even though it is slated for removal in rand 0.10.
    let mut rng = StdRng::seed_from_u64(42); // Fixed seed for reproducibility
    let time_needed = rng.gen_range(1..=10);

    // Inline formatting style per clippy.
    info!("Processing will take {time_needed}s");

    tokio::time::sleep(tokio::time::Duration::from_secs(time_needed)).await;

    Ok(())
}
