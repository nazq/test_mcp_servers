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
#[derive(Debug, Deserialize)]
pub struct RandomUuidParams {}
super::empty_params_schema!(
    RandomUuidParams,
    "Parameters for the `random_uuid` tool (no parameters needed)."
);

/// Parameters for the `current_time` tool (no parameters needed).
#[derive(Debug, Deserialize)]
pub struct CurrentTimeParams {}
super::empty_params_schema!(
    CurrentTimeParams,
    "Parameters for the `current_time` tool (no parameters needed)."
);
