//! Activity that logs an info message with context, returning the same string.
use log::info;
use temporal_sdk::{ActContext, ActivityError};

pub async fn log_activity(_ctx: ActContext, msg: String) -> Result<String, ActivityError> {
    info!(target: "custom_logger", "Activity logging message: {msg}");
    Ok(msg)
}
