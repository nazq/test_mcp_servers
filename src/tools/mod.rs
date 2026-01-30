//! Tool implementations for the MCP test server.
//!
//! This module provides 25 tools for comprehensive testing of MCP clients,
//! organized into the following categories:
//!
//! - **math**: Basic arithmetic operations (add, subtract, multiply, divide)
//! - **string**: Text manipulation (echo, concat, uppercase, lowercase, reverse, length)
//! - **encoding**: Data encoding/decoding (`json_parse`, `json_stringify`, `base64_encode`, `base64_decode`, `hash_sha256`)
//! - **utility**: Utility functions (`random_number`, `random_uuid`, `current_time`)
//! - **testing**: Testing helpers (sleep, fail, `fail_with_message`, `slow_echo`, `nested_data`, `large_response`, `binary_data`)

pub mod encoding;
pub mod math;
pub mod string;
pub mod testing;
pub mod ui;
pub mod utility;
