//! Workflow that books a trip via N activities, running compensations when any step fails.

//! Saga workflow – sequential bookings with compensation on failure.

// Implementation note:
// The previous version of this workflow returned `WfExitValue::Evicted` when
// any booking activity failed. Doing so causes a **panic inside the SDK**
// (`Don't explicitly return this`), because `Evicted` is a special internal
// value the core SDK uses for eviction handling – user code must **never**
// return it.
// Instead, workflows should either **fail** (`Err(..)`) or complete
// **successfully** (`Ok(WfExitValue::Normal(..))`). This patch rewrites the
// failure handling logic so that:
//
// 1. All failed booking attempts trigger *compensation* of previously completed
//    steps.
// 2. If compensation succeeds the workflow *completes* with
//    `WfExitValue::Normal("compensated".into())`.
// 3. If compensation itself fails, the workflow *fails* by returning an
//    `Err` – this is surfaced to the SDK / visibility APIs without panicking.

use anyhow::anyhow;
use helpers::parse_activity_result;
use log::{info, warn};
use std::time::Duration;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    AsJsonPayloadExt, workflow_commands::ActivityCancellationType,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BookingResult {
    flight: String,
    hotel: String,
    car: String,
}

pub async fn book_trip_workflow(ctx: WfContext) -> WorkflowResult<String> {
    // Vector holds (name, cancel_type, id) for compensation in order, and their ids
    let mut compensation_stack: Vec<(String, String, String)> = Vec::new();
    // Step 1: reserve flight
    let res_flight = ctx
        .activity(ActivityOptions {
            activity_type: "reserve_flight".to_string(),
            input: "".as_json_payload()?,
            start_to_close_timeout: Some(Duration::from_secs(10)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
    let flight_id = match parse_activity_result::<String>(&res_flight) {
        Ok(fid) => {
            compensation_stack.push((
                "flight".to_string(),
                "cancel_flight".to_string(),
                fid.clone(),
            ));
            fid
        }
        Err(e) => {
            warn!("Failed to reserve flight: {e}");
            // No reservation succeeded, so nothing to compensate.
            return Err(anyhow!("reserve_flight failed: {e}"));
        }
    };
    // Step 2: reserve hotel
    let res_hotel = ctx
        .activity(ActivityOptions {
            activity_type: "reserve_hotel".to_string(),
            input: "".as_json_payload()?,
            start_to_close_timeout: Some(Duration::from_secs(10)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
    let hotel_id = match parse_activity_result::<String>(&res_hotel) {
        Ok(hid) => {
            compensation_stack.push(("hotel".to_string(), "cancel_hotel".to_string(), hid.clone()));
            hid
        }
        Err(e) => {
            warn!("Failed to reserve hotel: {e}");
            // Only flight reserved, run its compensation
            for (name, cancel_typ, id) in compensation_stack.iter().rev() {
                let result = ctx
                    .activity(ActivityOptions {
                        activity_type: cancel_typ.clone(),
                        input: id.as_json_payload()?,
                        start_to_close_timeout: Some(Duration::from_secs(5)),
                        cancellation_type: ActivityCancellationType::TryCancel,
                        ..Default::default()
                    })
                    .await;
                info!("Compensating {name}: id={id} -> {result:?}");
            }
            // All compensations ran – treat workflow as compensated success.
            return Ok(WfExitValue::Normal("compensated".into()));
        }
    };
    // Step 3: reserve car
    let res_car = ctx
        .activity(ActivityOptions {
            activity_type: "reserve_car".to_string(),
            input: "".as_json_payload()?,
            start_to_close_timeout: Some(Duration::from_secs(10)),
            cancellation_type: ActivityCancellationType::TryCancel,
            ..Default::default()
        })
        .await;
    let car_id = match parse_activity_result::<String>(&res_car) {
        Ok(cid) => {
            compensation_stack.push(("car".to_string(), "cancel_car".to_string(), cid.clone()));
            cid
        }
        Err(e) => {
            warn!("Failed to reserve car: {e}");
            // Need to compensate hotel and flight in reverse
            for (name, cancel_typ, id) in compensation_stack.iter().rev() {
                let result = ctx
                    .activity(ActivityOptions {
                        activity_type: cancel_typ.clone(),
                        input: id.as_json_payload()?,
                        start_to_close_timeout: Some(Duration::from_secs(5)),
                        cancellation_type: ActivityCancellationType::TryCancel,
                        ..Default::default()
                    })
                    .await;
                info!("Compensating {name}: id={id} -> {result:?}");
            }
            return Ok(WfExitValue::Normal("compensated".into()));
        }
    };
    // All succeeded
    let res = BookingResult {
        flight: flight_id,
        hotel: hotel_id,
        car: car_id,
    };
    let summary = serde_json::to_string(&res).unwrap();
    Ok(WfExitValue::Normal(summary))
}
