//! Workflow for setting a custom search attribute and demonstrating upsert_search_attributes.
use log::info;
use temporal_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};

/// Workflow expecting a String arg (customer_id).
/// - Sets search attribute "CustomerId" to that value using upsert_search_attributes.
/// - Sleeps 2 seconds, then returns.
pub async fn set_search_attribute(ctx: WfContext) -> WorkflowResult<()> {
    // Get the customer_id arg
    let customer_id: String = match ctx
        .get_args()
        .first()
        .map(String::from_json_payload)
        .transpose()?  // Option<Result<T>> -> Result<Option<T>>
    {
        Some(val) => val,
        None => "unknown-customer".to_string(),
    };

    // NOTE: The Temporal dev-server ships with 8 pre-defined *dynamic* search attribute
    // keys named `CustomStringField`, `CustomIntField`, … etc.  Using one of those
    // avoids the “Namespace has no mapping defined” error you’d get when you attempt
    // to upsert a brand-new custom attribute such as `CustomerId` without first
    // registering it via `tctl admin cluster search-attribute add`.  For the sake of
    // this example we therefore use `CustomStringField`.

    info!("search_attributes: upserting CustomStringField={customer_id} (instead of custom key)");

    // Upsert search attribute with Temporal. The API expects a Map<String, Payload>.
    let payload = customer_id.as_json_payload()?;
    let mut search_attrs = std::collections::HashMap::new();
    search_attrs.insert("CustomStringField".to_string(), payload);

    ctx.upsert_search_attributes(search_attrs);

    // Wait 2 seconds (demonstrate workflow running so we can observe with open workflows filter)
    ctx.timer(std::time::Duration::from_secs(2)).await;

    Ok(WfExitValue::Normal(()))
}
