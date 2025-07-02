//! Workflow that books a trip via N activities, running compensations when any step fails.

use helpers::parse_activity_result;
use log::{info, warn};
use std::time::Duration;
use temporal_sdk::{ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    workflow_commands::ActivityCancellationType, AsJsonPayloadExt,
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
            return Ok(WfExitValue::Evicted);
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
            return Ok(WfExitValue::Evicted);
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
            return Ok(WfExitValue::Evicted);
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
