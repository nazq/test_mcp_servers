//! Math operation tools: add, subtract, multiply, divide.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for the add tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddParams {
    /// First number to add
    pub a: f64,
    /// Second number to add
    pub b: f64,
}

/// Parameters for the subtract tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubtractParams {
    /// Number to subtract from
    pub a: f64,
    /// Number to subtract
    pub b: f64,
}

/// Parameters for the multiply tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MultiplyParams {
    /// First number to multiply
    pub a: f64,
    /// Second number to multiply
    pub b: f64,
}

/// Parameters for the divide tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DivideParams {
    /// Numerator
    pub a: f64,
    /// Denominator
    pub b: f64,
}
