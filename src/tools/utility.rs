//! Utility tools: `random_number`, `random_uuid`, `current_time`.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for the `random_number` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RandomNumberParams {
    /// Minimum value (inclusive)
    pub min: i64,
    /// Maximum value (inclusive)
    pub max: i64,
}

/// Parameters for the `random_uuid` tool (no parameters needed).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RandomUuidParams {}

/// Parameters for the `current_time` tool (no parameters needed).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CurrentTimeParams {}
