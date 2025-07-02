//! Activities for the expense example: create, HTTP post, complete.
use log::info;
use serde::{Deserialize, Serialize};
use temporal_sdk::{ActContext, ActivityError};

/// Basic expense request info.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Expense {
    pub request_id: String,
    pub amount: f64,
    pub requester: String,
    pub description: String,
    pub status: String, // "pending", "approved", "rejected", "timed_out", etc
}

/// Activity that creates a new expense and returns it
pub async fn create_expense(
    _ctx: ActContext,
    input: Option<String>, // JSON: {amount, requester, description}
) -> Result<Expense, ActivityError> {
    let args: serde_json::Value = serde_json::from_str(&input.unwrap_or_default())
        .map_err(|e| ActivityError::NonRetryable(e.into()))?;
    let expense = Expense {
        request_id: uuid::Uuid::new_v4().to_string(),
        amount: args["amount"].as_f64().unwrap_or(0.0),
        requester: args["requester"].as_str().unwrap_or("").to_string(),
        description: args["description"].as_str().unwrap_or("").to_string(),
        status: "pending".into(),
    };
    info!("Created expense: {expense:?}");
    Ok(expense)
}

/// Dummy activity: would POST to external payment/HR server; here we log
pub async fn http_post(_ctx: ActContext, expense: Expense) -> Result<(), ActivityError> {
    info!("(HTTP POST) notifying server: expense={expense:?}");
    Ok(())
}

/// Activity that marks an expense as completed, for demo.
pub async fn complete_expense(
    _ctx: ActContext,
    mut expense: Expense,
) -> Result<Expense, ActivityError> {
    expense.status = match expense.status.as_str() {
        "approved" => "completed",
        x => x, // pass through for demo if not approved
    }
    .into();
    info!("Completing expense: {expense:?}");
    Ok(expense)
}
