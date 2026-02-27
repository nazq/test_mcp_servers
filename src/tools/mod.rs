//! Tool implementations for the MCP test server.
//!
//! This module provides 30 tools for comprehensive testing of MCP clients,
//! organized into the following categories:
//!
//! - **math**: Basic arithmetic operations (add, subtract, multiply, divide)
//! - **string**: Text manipulation (echo, concat, uppercase, lowercase, reverse, length)
//! - **encoding**: Data encoding/decoding (`json_parse`, `json_stringify`, `base64_encode`, `base64_decode`, `hash_sha256`)
//! - **utility**: Utility functions (`random_number`, `random_uuid`, `current_time`)
//! - **testing**: Testing helpers (sleep, fail, `fail_with_message`, `slow_echo`, `nested_data`, `large_response`, `binary_data`, noop)
//! - **ui**: MCP App interactive tools (`ui_resource_button`, `ui_resource_form`, `ui_resource_carousel`, `ui_internal_only`)

/// Generate a `JsonSchema` impl for an empty params struct that includes
/// `"properties": {}` in the output. VS Code Copilot Chat requires this field
/// even when the tool takes no parameters.
macro_rules! empty_params_schema {
    ($ty:ident, $desc:expr) => {
        impl schemars::JsonSchema for $ty {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                stringify!($ty).into()
            }

            fn schema_id() -> std::borrow::Cow<'static, str> {
                concat!(module_path!(), "::", stringify!($ty)).into()
            }

            fn json_schema(
                _gen: &mut schemars::SchemaGenerator,
            ) -> schemars::Schema {
                schemars::json_schema!({
                    "type": "object",
                    "title": stringify!($ty),
                    "description": $desc,
                    "properties": {}
                })
            }
        }
    };
}
pub(crate) use empty_params_schema;

pub mod encoding;
pub mod math;
pub mod string;
pub mod testing;
pub mod ui;
pub mod utility;
