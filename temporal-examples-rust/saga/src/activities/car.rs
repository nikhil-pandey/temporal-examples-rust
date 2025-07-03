//! Car reservation and cancellation activities for Saga example.

#![allow(clippy::uninlined_format_args)]

use log::info;
#[allow(deprecated)]
use rand::Rng;
use temporal_sdk::{ActContext, ActivityError};
use uuid::Uuid;

/// Simulate reserving a car. Randomly fails about 50% of the time.
pub async fn reserve_car(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<String, ActivityError> {
    // Generate a boolean that is `true` ~50 % of the time.
    let mut rng = rand::rng();
    let should_fail: bool = rng.random_bool(0.5);
    let id = Uuid::new_v4().to_string();
    info!("Trying to reserve car, simulated id={id}, will_fail={should_fail}");
    if should_fail {
        info!("Car reservation failed (simulated failure)");
        Err(ActivityError::NonRetryable(anyhow::anyhow!(
            "Car reservation failed"
        )))
    } else {
        info!("Car reserved successfully: {id}");
        Ok(id)
    }
}

/// Cancel reserved car, always succeeds.
pub async fn cancel_car(_ctx: ActContext, id: String) -> Result<(), ActivityError> {
    info!("Cancelling car with id={id}");
    Ok(())
}
