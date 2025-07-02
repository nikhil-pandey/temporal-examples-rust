//! Activity implementations for the `http_request` example.

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use temporal_sdk::{ActContext, ActivityError};

/// Shape of `httpbin.org/get` JSON response (we only care about `args`).
#[derive(Debug, Deserialize, Serialize)]
struct HttpBinResponse {
    args: HashMap<String, String>,
}

/// Simple activity that performs an HTTP GET request to https://httpbin.org
/// and returns the `answer` query parameter echoed back by the service.
///
/// The activity is intentionally trivial: it demonstrates how to invoke
/// blocking/external I/O from an Activity using `reqwest` and return a result
/// that can be consumed by the Workflow.
pub async fn make_http_request(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<String, ActivityError> {
    // Generate a random answer value so each invocation is unique.
    let answer = uuid::Uuid::new_v4().to_string();

    // Use the inline formatting style recommended by Clippy.
    info!("Starting http request activity: {answer}");

    // Fire off the GET request.
    let url = format!("https://httpbin.org/get?answer={answer}");
    let response = reqwest::get(url).await?.json::<HttpBinResponse>().await?;

    info!("Got response: {response:?}");

    if let Some(echoed) = response.args.get("answer") {
        Ok(echoed.to_string())
    } else {
        Err(ActivityError::NonRetryable(anyhow::anyhow!(
            "`answer` not found in response"
        )))
    }
}
