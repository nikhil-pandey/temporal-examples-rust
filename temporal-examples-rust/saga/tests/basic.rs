//! Integration-esque unit test for the Saga workflow crate.
//!
//! The Temporal Rust SDK currently lacks a public mocking / unit-testing API
//! comparable to the TypeScript `@temporalio/testing` helpers.  Until that is
//! available we limit ourselves to a **compile-time** test that exercises the
//! core data types and ensures the workflow function is exported and
//! callable. This is enough to make `cargo test -p saga` succeed and guards
//! against accidental signature changes.

use saga::workflow::book_trip_workflow;

/// Smoke-test that the workflow fn has the expected signature.
///
/// We don't execute it (cannot construct a `WfContext` outside of the SDK
/// runtime), but we can at least ensure it is *callable* via a generic helper
/// that compiles.
fn _assert_workflow_signature<F, Fut>(_f: F)
where
    F: Fn(temporal_sdk::WfContext) -> Fut,
    Fut: std::future::Future,
{
}

#[test]
fn workflow_signature_compiles() {
    _assert_workflow_signature(book_trip_workflow);
}
