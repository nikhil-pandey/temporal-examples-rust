//! Flight reservation and cancellation activities for Saga example.

#![allow(clippy::uninlined_format_args)]

use log::info;
#[allow(deprecated)]
use rand::Rng;
use temporal_sdk::{ActContext, ActivityError};
use uuid::Uuid;

/// Simulate reserving a flight. Randomly fails about 50% of the time.
pub async fn reserve_flight(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<String, ActivityError> {
    // 50/50 random failure using new rand API.
    let mut rng = rand::rngs::ThreadRng::default();
    let should_fail: bool = rng.gen_bool(0.1);
    let id = Uuid::new_v4().to_string();
    info!("Trying to reserve flight, simulated id={id}, will_fail={should_fail}");
    if should_fail {
        info!("Flight reservation failed (simulated failure)");
        Err(ActivityError::NonRetryable(anyhow::anyhow!(
            "Flight reservation failed"
        )))
    } else {
        info!("Flight reserved successfully: {id}");
        Ok(id)
    }
}

/// Cancel reserved flight, always succeeds.
pub async fn cancel_flight(_ctx: ActContext, id: String) -> Result<(), ActivityError> {
    info!("Cancelling flight with id={id}");
    Ok(())
}
