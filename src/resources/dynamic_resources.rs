//! Dynamic resources: counter, timestamp, random.

use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use rmcp::model::{AnnotateAble, RawResource, Resource, ResourceContents};

/// Counter state for the counter resource.
/// This is shared across all reads and increments on each access.
#[derive(Debug)]
pub struct CounterState {
    counter: AtomicU64,
}

impl CounterState {
    /// Create a new counter starting at 0.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }

    /// Increment and get the new value.
    pub fn increment(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst) + 1
    }
}

impl Default for CounterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the counter dynamic resource.
#[must_use]
pub fn get_counter_resource() -> Resource {
    RawResource {
        uri: "test://dynamic/counter".to_string(),
        name: "counter".to_string(),
        title: Some("Incrementing Counter".to_string()),
        description: Some("A counter that increments on each read".to_string()),
        mime_type: Some("text/plain".to_string()),
        size: None,
        icons: None,
    }
    .no_annotation()
}

/// Get the counter content with the given value.
#[must_use]
pub fn get_counter_content(value: u64) -> ResourceContents {
    ResourceContents::TextResourceContents {
        uri: "test://dynamic/counter".to_string(),
        mime_type: Some("text/plain".to_string()),
        text: format!("Counter value: {value}"),
        meta: None,
    }
}

/// Get the timestamp dynamic resource.
#[must_use]
pub fn get_timestamp_resource() -> Resource {
    RawResource {
        uri: "test://dynamic/timestamp".to_string(),
        name: "timestamp".to_string(),
        title: Some("Current Timestamp".to_string()),
        description: Some("Current UTC timestamp".to_string()),
        mime_type: Some("text/plain".to_string()),
        size: None,
        icons: None,
    }
    .no_annotation()
}

/// Get the timestamp content with current time.
#[must_use]
pub fn get_timestamp_content() -> ResourceContents {
    let now = Utc::now();
    ResourceContents::TextResourceContents {
        uri: "test://dynamic/timestamp".to_string(),
        mime_type: Some("text/plain".to_string()),
        text: format!("Current time: {}", now.to_rfc3339()),
        meta: None,
    }
}

/// Get the random dynamic resource.
#[must_use]
pub fn get_random_resource() -> Resource {
    RawResource {
        uri: "test://dynamic/random".to_string(),
        name: "random".to_string(),
        title: Some("Random Data".to_string()),
        description: Some("Random data for subscription testing".to_string()),
        mime_type: Some("text/plain".to_string()),
        size: None,
        icons: None,
    }
    .no_annotation()
}

/// Get random content.
#[must_use]
pub fn get_random_content() -> ResourceContents {
    use rand::Rng;
    let mut rng = rand::rng();
    let random_number: u64 = rng.random();
    let random_float: f64 = rng.random();

    ResourceContents::TextResourceContents {
        uri: "test://dynamic/random".to_string(),
        mime_type: Some("text/plain".to_string()),
        text: format!("Random number: {random_number}\nRandom float: {random_float:.10}"),
        meta: None,
    }
}

/// Get all dynamic resources.
#[must_use]
pub fn list_dynamic_resources() -> Vec<Resource> {
    vec![
        get_counter_resource(),
        get_timestamp_resource(),
        get_random_resource(),
    ]
}
