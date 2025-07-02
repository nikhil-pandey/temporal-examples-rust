//! Demonstrates activities using dependency-injected Greeter services.
use std::sync::Arc;
use temporal_sdk::{ActContext, ActivityError};

use once_cell::sync::OnceCell;

/// Greeter service that owns a language tag (&str).
#[derive(Debug)]
pub struct Greeter {
    lang: &'static str,
}

impl Greeter {
    pub fn new(lang: &'static str) -> Self {
        Self { lang }
    }
    pub fn greet(&self, name: &str) -> String {
        match self.lang {
            "en" => format!("Hello, {name}!"),
            "es" => format!("Hola, {name}!"),
            other => format!("({other}) Hi, {name}!"),
        }
    }
}

// Global per-activity statics, set at worker startup (initialized in main)
pub static EN_GREETER: OnceCell<Arc<Greeter>> = OnceCell::new();
pub static ES_GREETER: OnceCell<Arc<Greeter>> = OnceCell::new();

/// English greeting activity, uses EN_GREETER
pub async fn greet_english(_ctx: ActContext, name: String) -> Result<String, ActivityError> {
    Ok(EN_GREETER
        .get()
        .expect("EN_GREETER not initialized")
        .greet(&name))
}

/// Spanish greeting activity, uses ES_GREETER
pub async fn greet_spanish(_ctx: ActContext, name: String) -> Result<String, ActivityError> {
    Ok(ES_GREETER
        .get()
        .expect("ES_GREETER not initialized")
        .greet(&name))
}
