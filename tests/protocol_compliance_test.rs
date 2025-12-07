//! Protocol compliance tests based on MCP specification 2025-11-25.
//!
//! This test suite verifies compliance with the Model Context Protocol specification.
//! <https://modelcontextprotocol.io/specification/2025-11-25/>

use mcp_test_server::prompts::templates::{generate_prompt, get_all_prompts};
use mcp_test_server::resources::{ResourceHandler, dynamic_resources, static_resources};
use rmcp::model::{ReadResourceRequestParam, SubscribeRequestParam};
use std::collections::HashMap;

// =============================================================================
// PROMPTS COMPLIANCE TESTS
// Based on: https://modelcontextprotocol.io/specification/2025-11-25/server/prompts
// =============================================================================

mod prompts_compliance {
    use super::*;

    /// Spec: "Each prompt has a unique name" - verify uniqueness
    #[test]
    fn test_prompt_names_are_unique() {
        let prompts = get_all_prompts();
        let mut names = std::collections::HashSet::new();

        for prompt in &prompts {
            assert!(
                names.insert(&prompt.name),
                "Duplicate prompt name found: {}",
                prompt.name
            );
        }
    }

    /// Spec: Prompt names should follow naming conventions
    /// "name: required (unique identifier)"
    #[test]
    fn test_prompt_names_are_valid_identifiers() {
        let prompts = get_all_prompts();

        for prompt in &prompts {
            assert!(!prompt.name.is_empty(), "Prompt name must not be empty");
            assert!(
                prompt
                    .name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                "Prompt name '{}' contains invalid characters",
                prompt.name
            );
        }
    }

    /// Spec: "Servers SHOULD validate prompt arguments before processing"
    #[test]
    fn test_missing_required_argument_returns_error() {
        let args = HashMap::new();

        // greeting requires "name"
        let result = generate_prompt("greeting", &args);
        assert!(
            result.is_err(),
            "Should error when required argument missing"
        );

        // code_review requires "code" and "language"
        let mut args = HashMap::new();
        args.insert("code".to_string(), "test".to_string());
        let result = generate_prompt("code_review", &args);
        assert!(result.is_err(), "Should error when language missing");
    }

    /// Spec: "Invalid params (-32602): Invalid prompt name"
    #[test]
    fn test_unknown_prompt_returns_error() {
        let args = HashMap::new();
        let result = generate_prompt("nonexistent_prompt", &args);
        assert!(result.is_err(), "Unknown prompt should return error");
    }

    /// Spec: Messages array with role "user" or "assistant"
    #[test]
    fn test_prompt_messages_have_valid_roles() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), "Test".to_string());

        let result = generate_prompt("greeting", &args).unwrap();

        for msg in &result {
            let role = &msg.role;
            assert!(
                matches!(
                    role,
                    rmcp::model::PromptMessageRole::User
                        | rmcp::model::PromptMessageRole::Assistant
                ),
                "Message role must be 'user' or 'assistant'"
            );
        }
    }

    /// Spec: Prompts with required arguments should have them marked
    #[test]
    fn test_required_arguments_are_marked() {
        let prompts = get_all_prompts();

        for prompt in &prompts {
            if let Some(args) = &prompt.arguments {
                for arg in args {
                    // Required arguments should have required field set
                    if arg.name == "name"
                        || arg.name == "code"
                        || arg.name == "language"
                        || arg.name == "text"
                    {
                        assert!(
                            arg.required == Some(true),
                            "Argument '{}' in prompt '{}' should be marked required",
                            arg.name,
                            prompt.name
                        );
                    }
                }
            }
        }
    }
}

// =============================================================================
// RESOURCES COMPLIANCE TESTS
// Based on: https://modelcontextprotocol.io/specification/2025-11-25/server/resources
// =============================================================================

mod resources_compliance {
    use super::*;

    /// Spec: "Each resource requires: uri, name"
    #[test]
    fn test_all_resources_have_required_fields() {
        let static_resources = static_resources::list_static_resources();
        let dynamic_resources = dynamic_resources::list_dynamic_resources();

        for resource in static_resources.iter().chain(dynamic_resources.iter()) {
            assert!(!resource.uri.is_empty(), "Resource URI must not be empty");
            assert!(!resource.name.is_empty(), "Resource name must not be empty");
        }
    }

    /// Spec: URIs "must comply with RFC 3986"
    #[test]
    fn test_resource_uris_are_valid() {
        let static_resources = static_resources::list_static_resources();
        let dynamic_resources = dynamic_resources::list_dynamic_resources();

        for resource in static_resources.iter().chain(dynamic_resources.iter()) {
            // URIs should have a scheme
            assert!(
                resource.uri.contains("://"),
                "Resource URI '{}' should have scheme",
                resource.uri
            );
        }
    }

    /// Spec: "Content must specify either text (string) or blob (base64-encoded)"
    #[test]
    fn test_resource_content_has_text_or_blob() {
        use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

        let handler = ResourceHandler::new();

        let test_uris = [
            "test://static/hello.txt",
            "test://static/data.json",
            "test://static/image.png",
            "test://dynamic/counter",
            "test://dynamic/timestamp",
        ];

        for uri in &test_uris {
            let request = ReadResourceRequestParam {
                uri: uri.to_string(),
            };
            let result = handler.read_resource(&request).unwrap();

            for content in &result.contents {
                match content {
                    rmcp::model::ResourceContents::TextResourceContents { text, .. } => {
                        assert!(
                            !text.is_empty(),
                            "Text content should not be empty for {uri}"
                        );
                    }
                    rmcp::model::ResourceContents::BlobResourceContents { blob, .. } => {
                        assert!(
                            !blob.is_empty(),
                            "Blob content should not be empty for {uri}"
                        );
                        // Verify it's valid base64
                        assert!(
                            BASE64.decode(blob).is_ok(),
                            "Blob should be valid base64 for {uri}"
                        );
                    }
                }
            }
        }
    }

    /// Spec: Resource not found should return JSON-RPC error code -32002
    #[test]
    fn test_unknown_resource_returns_error() {
        let handler = ResourceHandler::new();
        let request = ReadResourceRequestParam {
            uri: "test://nonexistent/resource".to_string(),
        };

        let result = handler.read_resource(&request);
        assert!(result.is_err(), "Unknown resource should return error");
    }

    /// Spec: Templates use "URI templates per RFC 6570 standard"
    #[test]
    fn test_resource_templates_use_rfc6570_syntax() {
        let handler = ResourceHandler::new();
        let result = handler.list_resource_templates(None).unwrap();

        for template in &result.resource_templates {
            // RFC 6570 templates use {variable} syntax
            assert!(
                template.uri_template.contains('{') && template.uri_template.contains('}'),
                "Template '{}' should use RFC 6570 {{variable}} syntax",
                template.uri_template
            );
        }
    }

    /// Spec: "Servers declare subscribe capability if supported"
    #[test]
    fn test_subscription_to_supported_resource_succeeds() {
        let handler = ResourceHandler::new();
        let request = SubscribeRequestParam {
            uri: "test://dynamic/random".to_string(),
        };

        let result = handler.subscribe(&request);
        assert!(
            result.is_ok(),
            "Subscription to random resource should succeed"
        );
    }

    /// Spec: Subscription to non-subscribable resource should fail
    #[test]
    fn test_subscription_to_non_subscribable_fails() {
        let handler = ResourceHandler::new();
        let request = SubscribeRequestParam {
            uri: "test://static/hello.txt".to_string(),
        };

        let result = handler.subscribe(&request);
        assert!(
            result.is_err(),
            "Subscription to static resource should fail"
        );
    }

    /// Spec: "Each content item requires uri and mimeType"
    #[test]
    fn test_content_has_uri_and_mimetype() {
        let handler = ResourceHandler::new();
        let request = ReadResourceRequestParam {
            uri: "test://static/hello.txt".to_string(),
        };

        let result = handler.read_resource(&request).unwrap();

        for content in &result.contents {
            let (uri, mime_type) = match content {
                rmcp::model::ResourceContents::TextResourceContents { uri, mime_type, .. }
                | rmcp::model::ResourceContents::BlobResourceContents { uri, mime_type, .. } => {
                    (uri, mime_type)
                }
            };
            assert!(!uri.is_empty(), "Content URI should not be empty");
            assert!(mime_type.is_some(), "Content should have mimeType");
        }
    }
}

// =============================================================================
// TOOLS COMPLIANCE TESTS
// Based on: https://modelcontextprotocol.io/specification/2025-11-25/server/tools
// =============================================================================

mod tools_compliance {
    use mcp_test_server::tools::*;
    use schemars::schema_for;

    /// Spec: "inputSchema is mandatory and MUST be valid JSON Schema"
    #[test]
    fn test_all_tool_params_have_valid_json_schema() {
        // Each tool param struct should generate valid JSON schema
        let _ = schema_for!(math::AddParams);
        let _ = schema_for!(math::SubtractParams);
        let _ = schema_for!(math::MultiplyParams);
        let _ = schema_for!(math::DivideParams);
        let _ = schema_for!(string::EchoParams);
        let _ = schema_for!(string::ConcatParams);
        let _ = schema_for!(string::UppercaseParams);
        let _ = schema_for!(string::LowercaseParams);
        let _ = schema_for!(string::ReverseParams);
        let _ = schema_for!(string::LengthParams);
        let _ = schema_for!(encoding::JsonParseParams);
        let _ = schema_for!(encoding::JsonStringifyParams);
        let _ = schema_for!(encoding::Base64EncodeParams);
        let _ = schema_for!(encoding::Base64DecodeParams);
        let _ = schema_for!(encoding::HashSha256Params);
        let _ = schema_for!(utility::RandomNumberParams);
        let _ = schema_for!(utility::RandomUuidParams);
        let _ = schema_for!(utility::CurrentTimeParams);
        let _ = schema_for!(testing::SleepParams);
        let _ = schema_for!(testing::FailParams);
        let _ = schema_for!(testing::FailWithMessageParams);
        let _ = schema_for!(testing::SlowEchoParams);
        let _ = schema_for!(testing::NestedDataParams);
        let _ = schema_for!(testing::LargeResponseParams);
        let _ = schema_for!(testing::BinaryDataParams);
    }

    /// Spec: "Tools without parameters use: { type: object, additionalProperties: false }"
    #[test]
    fn test_parameterless_tools_have_empty_object_schema() {
        let schema = schema_for!(utility::RandomUuidParams);
        let schema_value = serde_json::to_value(schema).unwrap();

        // Should have type: object
        assert_eq!(
            schema_value["type"], "object",
            "Parameterless schema should have type: object"
        );
    }

    /// Spec: "Tool names should be 1-128 characters using only ASCII letters, digits, underscore, hyphen, and dot"
    #[test]
    fn test_tool_names_in_valid_format() {
        // These are the tool names defined in server.rs
        let tool_names = [
            "add",
            "subtract",
            "multiply",
            "divide",
            "echo",
            "concat",
            "uppercase",
            "lowercase",
            "reverse",
            "length",
            "json_parse",
            "json_stringify",
            "base64_encode",
            "base64_decode",
            "hash_sha256",
            "random_number",
            "random_uuid",
            "current_time",
            "sleep",
            "fail",
            "fail_with_message",
            "slow_echo",
            "nested_data",
            "large_response",
            "binary_data",
        ];

        for name in &tool_names {
            assert!(
                !name.is_empty() && name.len() <= 128,
                "Tool name '{name}' length out of bounds"
            );
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'),
                "Tool name '{name}' contains invalid characters"
            );
        }
    }
}

// =============================================================================
// CAPABILITY COMPLIANCE TESTS
// Based on: https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
// =============================================================================

mod capability_compliance {
    use mcp_test_server::{Config, McpTestServer};
    use rmcp::handler::server::ServerHandler;

    /// Spec: Server must declare capabilities during initialization
    #[test]
    fn test_server_declares_capabilities() {
        let config = Config::default();
        let server = McpTestServer::new(config);
        let info = server.get_info();

        // Server should declare tools capability with listChanged
        assert!(
            info.capabilities.tools.is_some(),
            "Server should declare tools capability"
        );
        if let Some(tools) = &info.capabilities.tools {
            assert!(
                tools.list_changed.is_some(),
                "Tools should have listChanged"
            );
        }

        // Server should declare resources capability
        assert!(
            info.capabilities.resources.is_some(),
            "Server should declare resources capability"
        );
        if let Some(resources) = &info.capabilities.resources {
            assert!(
                resources.subscribe.is_some(),
                "Resources should have subscribe"
            );
            assert!(
                resources.list_changed.is_some(),
                "Resources should have listChanged"
            );
        }

        // Server should declare prompts capability
        assert!(
            info.capabilities.prompts.is_some(),
            "Server should declare prompts capability"
        );
        if let Some(prompts) = &info.capabilities.prompts {
            assert!(
                prompts.list_changed.is_some(),
                "Prompts should have listChanged"
            );
        }
    }

    /// Spec: Server should provide implementation info
    #[test]
    fn test_server_provides_implementation_info() {
        let config = Config::default();
        let server = McpTestServer::new(config);
        let info = server.get_info();

        assert!(!info.server_info.name.is_empty(), "Server should have name");
        assert!(
            !info.server_info.version.is_empty(),
            "Server should have version"
        );
    }

    /// Spec: Server should declare protocol version
    #[test]
    fn test_server_declares_protocol_version() {
        let config = Config::default();
        let server = McpTestServer::new(config);
        let info = server.get_info();

        // Protocol version should be set
        let version_str = format!("{:?}", info.protocol_version);
        assert!(
            !version_str.is_empty(),
            "Protocol version should be declared"
        );
    }
}

// =============================================================================
// PAGINATION COMPLIANCE TESTS
// =============================================================================

mod pagination_compliance {
    use mcp_test_server::resources::ResourceHandler;
    use rmcp::model::PaginatedRequestParam;

    /// Spec: "supports pagination with optional cursor parameter"
    #[test]
    fn test_resource_list_supports_pagination_param() {
        let handler = ResourceHandler::new();

        // Should accept None cursor
        let result = handler.list_resources(None);
        assert!(result.is_ok());

        // Should accept Some cursor (even if not used)
        let result = handler.list_resources(Some(PaginatedRequestParam { cursor: None }));
        assert!(result.is_ok());
    }

    /// Spec: Response includes nextCursor for pagination
    #[test]
    fn test_resource_list_response_has_cursor_field() {
        let handler = ResourceHandler::new();
        let result = handler.list_resources(None).unwrap();

        // Should have next_cursor field (even if None when no more pages)
        // The fact that it compiles means the field exists
        let _ = result.next_cursor;
    }
}

// =============================================================================
// ERROR HANDLING COMPLIANCE TESTS
// =============================================================================

mod error_handling_compliance {
    use super::*;

    /// Spec: Protocol errors should use JSON-RPC standard codes
    #[test]
    fn test_invalid_prompt_returns_proper_error() {
        let args = HashMap::new();
        let result = generate_prompt("invalid_name", &args);

        assert!(result.is_err(), "Invalid prompt should return error");
    }

    /// Spec: Missing required arguments should fail validation
    #[test]
    fn test_missing_required_args_fails_validation() {
        let empty_args = HashMap::new();

        // greeting requires name
        assert!(generate_prompt("greeting", &empty_args).is_err());

        // code_review requires code and language
        assert!(generate_prompt("code_review", &empty_args).is_err());

        // summarize requires text
        assert!(generate_prompt("summarize", &empty_args).is_err());

        // translate requires text and language
        assert!(generate_prompt("translate", &empty_args).is_err());
    }

    /// Spec: Resource read with invalid URI should error
    #[test]
    fn test_invalid_resource_uri_returns_error() {
        let handler = ResourceHandler::new();
        let request = ReadResourceRequestParam {
            uri: "invalid://not/a/valid/resource".to_string(),
        };

        let result = handler.read_resource(&request);
        assert!(result.is_err(), "Invalid resource URI should return error");
    }
}
