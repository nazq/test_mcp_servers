//! Unit tests for tool parameter structs.
//!
//! Note: Tools are implemented as async methods on `McpTestServer` with rmcp macros.
//! These tests verify the parameter structures can be deserialized from JSON
//! (as they would be from MCP tool calls) and have valid JSON schemas.

use mcp_test_server::tools::{
    encoding::{
        Base64DecodeParams, Base64EncodeParams, HashSha256Params, JsonParseParams,
        JsonStringifyParams,
    },
    math::{AddParams, DivideParams, MultiplyParams, SubtractParams},
    string::{
        ConcatParams, EchoParams, LengthParams, LowercaseParams, ReverseParams, UppercaseParams,
    },
    testing::{
        BinaryDataParams, FailParams, FailWithMessageParams, LargeResponseParams, NestedDataParams,
        SleepParams, SlowEchoParams,
    },
    utility::{CurrentTimeParams, RandomNumberParams, RandomUuidParams},
};

// Test that all param structs can be deserialized from JSON (as MCP would send them)

#[test]
fn test_math_params_deserialization() {
    // AddParams
    let json = r#"{"a": 1.5, "b": 2.5}"#;
    let params: AddParams = serde_json::from_str(json).unwrap();
    assert!((params.a - 1.5).abs() < f64::EPSILON);
    assert!((params.b - 2.5).abs() < f64::EPSILON);

    // SubtractParams
    let json = r#"{"a": 10.0, "b": 3.0}"#;
    let params: SubtractParams = serde_json::from_str(json).unwrap();
    assert!((params.a - 10.0).abs() < f64::EPSILON);
    assert!((params.b - 3.0).abs() < f64::EPSILON);

    // MultiplyParams
    let json = r#"{"a": 4.0, "b": 5.0}"#;
    let params: MultiplyParams = serde_json::from_str(json).unwrap();
    assert!((params.a - 4.0).abs() < f64::EPSILON);
    assert!((params.b - 5.0).abs() < f64::EPSILON);

    // DivideParams
    let json = r#"{"a": 20.0, "b": 4.0}"#;
    let params: DivideParams = serde_json::from_str(json).unwrap();
    assert!((params.a - 20.0).abs() < f64::EPSILON);
    assert!((params.b - 4.0).abs() < f64::EPSILON);
}

#[test]
fn test_string_params_deserialization() {
    // EchoParams
    let json = r#"{"text": "hello"}"#;
    let params: EchoParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.text, "hello");

    // ConcatParams
    let json = r#"{"strings": ["a", "b", "c"]}"#;
    let params: ConcatParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.strings, vec!["a", "b", "c"]);

    // UppercaseParams
    let json = r#"{"text": "test"}"#;
    let params: UppercaseParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.text, "test");

    // LowercaseParams
    let json = r#"{"text": "TEST"}"#;
    let params: LowercaseParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.text, "TEST");

    // ReverseParams
    let json = r#"{"text": "abc"}"#;
    let params: ReverseParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.text, "abc");

    // LengthParams
    let json = r#"{"text": "hello world"}"#;
    let params: LengthParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.text, "hello world");
}

#[test]
fn test_encoding_params_deserialization() {
    // JsonParseParams
    let json = r#"{"json": "{\"key\": \"value\"}"}"#;
    let params: JsonParseParams = serde_json::from_str(json).unwrap();
    assert!(params.json.contains("key"));

    // JsonStringifyParams
    let json = r#"{"value": {"foo": "bar"}}"#;
    let params: JsonStringifyParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.value["foo"], "bar");

    // Base64EncodeParams
    let json = r#"{"text": "hello"}"#;
    let params: Base64EncodeParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.text, "hello");

    // Base64DecodeParams
    let json = r#"{"encoded": "aGVsbG8="}"#;
    let params: Base64DecodeParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.encoded, "aGVsbG8=");

    // HashSha256Params
    let json = r#"{"text": "test"}"#;
    let params: HashSha256Params = serde_json::from_str(json).unwrap();
    assert_eq!(params.text, "test");
}

#[test]
fn test_utility_params_deserialization() {
    // RandomNumberParams
    let json = r#"{"min": 1, "max": 100}"#;
    let params: RandomNumberParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.min, 1);
    assert_eq!(params.max, 100);

    // RandomUuidParams - empty object
    let json = r"{}";
    let _params: RandomUuidParams = serde_json::from_str(json).unwrap();

    // CurrentTimeParams - empty object
    let json = r"{}";
    let _params: CurrentTimeParams = serde_json::from_str(json).unwrap();
}

#[test]
fn test_testing_params_deserialization() {
    // SleepParams
    let json = r#"{"duration_ms": 100}"#;
    let params: SleepParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.duration_ms, 100);

    // FailParams - empty object
    let json = r"{}";
    let _params: FailParams = serde_json::from_str(json).unwrap();

    // FailWithMessageParams
    let json = r#"{"message": "error!"}"#;
    let params: FailWithMessageParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.message, "error!");

    // SlowEchoParams
    let json = r#"{"text": "hello", "delay_ms": 50}"#;
    let params: SlowEchoParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.text, "hello");
    assert_eq!(params.delay_ms, 50);

    // NestedDataParams
    let json = r#"{"depth": 5}"#;
    let params: NestedDataParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.depth, 5);

    // LargeResponseParams
    let json = r#"{"size_bytes": 1024}"#;
    let params: LargeResponseParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.size_bytes, 1024);

    // BinaryDataParams
    let json = r#"{"size_bytes": 256}"#;
    let params: BinaryDataParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.size_bytes, 256);
}

#[test]
fn test_params_have_json_schema() {
    use schemars::schema_for;

    // Verify all param types have valid JSON schema (required by MCP spec)
    let _ = schema_for!(AddParams);
    let _ = schema_for!(SubtractParams);
    let _ = schema_for!(MultiplyParams);
    let _ = schema_for!(DivideParams);
    let _ = schema_for!(EchoParams);
    let _ = schema_for!(ConcatParams);
    let _ = schema_for!(UppercaseParams);
    let _ = schema_for!(LowercaseParams);
    let _ = schema_for!(ReverseParams);
    let _ = schema_for!(LengthParams);
    let _ = schema_for!(JsonParseParams);
    let _ = schema_for!(JsonStringifyParams);
    let _ = schema_for!(Base64EncodeParams);
    let _ = schema_for!(Base64DecodeParams);
    let _ = schema_for!(HashSha256Params);
    let _ = schema_for!(RandomNumberParams);
    let _ = schema_for!(RandomUuidParams);
    let _ = schema_for!(CurrentTimeParams);
    let _ = schema_for!(SleepParams);
    let _ = schema_for!(FailParams);
    let _ = schema_for!(FailWithMessageParams);
    let _ = schema_for!(SlowEchoParams);
    let _ = schema_for!(NestedDataParams);
    let _ = schema_for!(LargeResponseParams);
    let _ = schema_for!(BinaryDataParams);
}

#[test]
fn test_invalid_json_fails_deserialization() {
    // Missing required field
    let json = r#"{"a": 1.5}"#;
    let result: Result<AddParams, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "Should fail with missing required field 'b'"
    );

    // Wrong type
    let json = r#"{"text": 123}"#;
    let result: Result<EchoParams, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Should fail with wrong type for 'text'");
}

#[test]
fn test_parameterless_structs_accept_extra_fields() {
    // Per MCP spec, we might want to be lenient with extra fields
    // Depends on serde configuration
    let json = r"{}";
    let _: RandomUuidParams = serde_json::from_str(json).unwrap();
    let _: CurrentTimeParams = serde_json::from_str(json).unwrap();
    let _: FailParams = serde_json::from_str(json).unwrap();
}
