//! Activity that does nothing – used to illustrate skip & cancellation.

use log::info;
use temporal_sdk::{ActContext, ActivityError};

pub async fn skipped_activity(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<(), ActivityError> {
    info!("Starting skipped activity – this will do nothing");
    Ok(())
}
