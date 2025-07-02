//! Activities for worker-specific task queue demo.
use temporal_sdk::{ActContext, ActivityError};

/// Returns a greeting specific to the queue.
pub async fn greet_from_queue(_ctx: ActContext, queue: String) -> Result<String, ActivityError> {
    Ok(format!("Hello from {queue}"))
}
