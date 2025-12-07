//! Encoding tools: `json_parse`, `json_stringify`, `base64_encode`, `base64_decode`, `hash_sha256`.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for the `json_parse` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JsonParseParams {
    /// JSON string to parse
    pub json: String,
}

/// Parameters for the `json_stringify` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct JsonStringifyParams {
    /// Value to convert to JSON
    pub value: serde_json::Value,
}

/// Parameters for the `base64_encode` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Base64EncodeParams {
    /// Text to encode
    pub text: String,
}

/// Parameters for the `base64_decode` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Base64DecodeParams {
    /// Base64 string to decode
    pub encoded: String,
}

/// Parameters for the `hash_sha256` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HashSha256Params {
    /// Text to hash
    pub text: String,
}
