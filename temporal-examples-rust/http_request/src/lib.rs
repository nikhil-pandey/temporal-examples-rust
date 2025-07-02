//! `http_request` example crate.
//!
//! This crate demonstrates a simple Activity that makes an outbound HTTP
//! request and returns the response data to the Workflow. It is intentionally
//! lightweight and mirrors the equivalent TypeScript sample that calls
//! `https://httpbin.org/get` with a random query parameter.

pub mod activities;
pub mod workflow;
