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
