//! Hotel reservation and cancellation activities for Saga example.

#![allow(clippy::uninlined_format_args)]

use log::info;
#[allow(deprecated)]
use rand::Rng;
use temporal_sdk::{ActContext, ActivityError};
use uuid::Uuid;

/// Simulate reserving a hotel. Randomly fails about 50% of the time.
pub async fn reserve_hotel(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<String, ActivityError> {
    let mut rng = rand::rng();
    let should_fail: bool = rng.random_bool(0.7);
    let id = Uuid::new_v4().to_string();
    info!("Trying to reserve hotel, simulated id={id}, will_fail={should_fail}");
    if should_fail {
        info!("Hotel reservation failed (simulated failure)");
        Err(ActivityError::NonRetryable(anyhow::anyhow!(
            "Hotel reservation failed"
        )))
    } else {
        info!("Hotel reserved successfully: {id}");
        Ok(id)
    }
}

/// Cancel reserved hotel, always succeeds.
pub async fn cancel_hotel(_ctx: ActContext, id: String) -> Result<(), ActivityError> {
    info!("Cancelling hotel with id={id}");
    Ok(())
}
