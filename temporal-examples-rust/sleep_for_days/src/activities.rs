//! Email activity for the sleep_for_days example.
use log::info;
use temporal_sdk::{ActContext, ActivityError};

/// Activity that logs a fake "email sent" message, taking any string payload.
pub async fn send_email(_ctx: ActContext, payload: Option<String>) -> Result<(), ActivityError> {
    info!(target: "sleep_for_days", "[send_email activity] Sent email: {payload:?}");
    Ok(())
}
