//! String operation tools: echo, concat, uppercase, lowercase, reverse, length.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for the echo tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EchoParams {
    /// Text to echo back
    pub text: String,
}

/// Parameters for the concat tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConcatParams {
    /// Strings to concatenate
    pub strings: Vec<String>,
}

/// Parameters for the uppercase tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UppercaseParams {
    /// Text to convert to uppercase
    pub text: String,
}

/// Parameters for the lowercase tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LowercaseParams {
    /// Text to convert to lowercase
    pub text: String,
}

/// Parameters for the reverse tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReverseParams {
    /// Text to reverse
    pub text: String,
}

/// Parameters for the length tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LengthParams {
    /// Text to get length of
    pub text: String,
}
