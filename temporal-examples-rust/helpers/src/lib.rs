//! Shared utilities used by multiple example crates.

use std::str::FromStr;

use anyhow::Context;
use temporal_client::{Client, RetryClient};
use temporal_sdk::sdk_client_options;
use temporal_sdk_core::Url;

/// Connect to the Temporal server running on `localhost:7233` with namespace
/// `default`.
///
/// This helper is intentionally minimal – it is only meant to avoid repeating
/// boilerplate in every example binary.
pub async fn get_client() -> Result<RetryClient<Client>, anyhow::Error> {
    // Build default client options that point at the local test server.
    let opts = sdk_client_options(Url::from_str("http://localhost:7233")?)
        .build()
        .context("failed building Temporal client options")?;

    // Connect returns a `RetryClient<Client>` which automatically retries
    // transient failures.
    let client = opts
        .connect("default", None)
        .await
        .context("failed connecting to Temporal server")?;

    Ok(client)
}

mod parse_activity_result;

pub use parse_activity_result::parse_activity_result;
