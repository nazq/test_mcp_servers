//! Testing tools: sleep, fail, `fail_with_message`, `slow_echo`, `nested_data`, `large_response`, `binary_data`.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for the sleep tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SleepParams {
    /// Duration to sleep in milliseconds
    pub duration_ms: u64,
}

/// Parameters for the fail tool (no parameters needed).
#[derive(Debug, Deserialize)]
pub struct FailParams {}
super::empty_params_schema!(
    FailParams,
    "Parameters for the fail tool (no parameters needed)."
);

/// Parameters for the `fail_with_message` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FailWithMessageParams {
    /// Error message to return
    pub message: String,
}

/// Parameters for the `slow_echo` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SlowEchoParams {
    /// Text to echo
    pub text: String,
    /// Delay in milliseconds before echoing
    pub delay_ms: u64,
}

/// Parameters for the `nested_data` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NestedDataParams {
    /// Depth of nesting
    pub depth: usize,
}

/// Parameters for the `large_response` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LargeResponseParams {
    /// Size of response in bytes (approximately)
    pub size_bytes: usize,
}

/// Parameters for the `binary_data` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BinaryDataParams {
    /// Size of binary data in bytes
    pub size_bytes: usize,
}

// =============================================================================
// TASK TOOLS — async long-running operations (MCP Tasks spec)
// =============================================================================

/// Parameters for `task_slow_compute` — a long-running operation that reports progress.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskSlowComputeParams {
    /// Duration of the computation in seconds (default: 5)
    #[serde(default = "default_task_duration")]
    pub duration_secs: u64,
}

/// Parameters for `task_cancellable` — a long-running operation that responds to cancellation.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskCancellableParams {
    /// Duration of the computation in seconds (default: 30)
    #[serde(default = "default_cancellable_duration")]
    pub duration_secs: u64,
}

/// Parameters for `task_fail` — starts a task that fails after a delay.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskFailParams {
    /// Seconds to wait before failing (default: 2)
    #[serde(default = "default_fail_duration")]
    pub duration_secs: u64,
    /// Error message to return on failure
    #[serde(default = "default_fail_message")]
    pub message: String,
}

const fn default_task_duration() -> u64 {
    5
}

const fn default_cancellable_duration() -> u64 {
    30
}

const fn default_fail_duration() -> u64 {
    2
}

fn default_fail_message() -> String {
    "Task failed as expected".to_string()
}
