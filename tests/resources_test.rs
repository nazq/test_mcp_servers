//! Integration tests for resources implementation.

use mcp_test_server::resources::{
    ResourceHandler,
    dynamic_resources::{
        CounterState, get_counter_content, get_counter_resource, get_random_content,
        get_random_resource, get_timestamp_content, get_timestamp_resource, list_dynamic_resources,
    },
    static_resources::{
        get_data_json_content, get_data_json_resource, get_hello_content, get_hello_resource,
        get_image_png_content, get_image_png_resource, get_large_txt_content,
        get_large_txt_resource, list_static_resources, read_static_resource,
    },
};
use rmcp::model::{ReadResourceRequestParams, ResourceContents, SubscribeRequestParams};

// Static resource tests

#[test]
fn test_list_static_resources() {
    let resources = list_static_resources();
    // 4 original static + 3 UI app resources = 7
    assert_eq!(resources.len(), 7);
}

#[test]
fn test_hello_resource() {
    let resource = get_hello_resource();
    assert_eq!(resource.uri, "test://static/hello.txt");
    assert_eq!(resource.name, "hello.txt");
    assert_eq!(resource.mime_type, Some("text/plain".to_string()));
}

#[test]
fn test_hello_content() {
    let content = get_hello_content();
    match content {
        ResourceContents::TextResourceContents { text, .. } => {
            assert_eq!(text, "Hello, World!");
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

#[test]
fn test_data_json_resource() {
    let resource = get_data_json_resource();
    assert_eq!(resource.uri, "test://static/data.json");
    assert_eq!(resource.mime_type, Some("application/json".to_string()));
}

#[test]
fn test_data_json_content_is_valid_json() {
    let content = get_data_json_content();
    match content {
        ResourceContents::TextResourceContents { text, .. } => {
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(parsed["name"], "test");
            assert_eq!(parsed["version"], "1.0");
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

#[test]
fn test_image_png_resource() {
    let resource = get_image_png_resource();
    assert_eq!(resource.uri, "test://static/image.png");
    assert_eq!(resource.mime_type, Some("image/png".to_string()));
}

#[test]
fn test_image_png_content_is_base64() {
    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

    let content = get_image_png_content();
    match content {
        ResourceContents::BlobResourceContents { blob, .. } => {
            // Should be valid base64
            let decoded = BASE64.decode(&blob);
            assert!(decoded.is_ok(), "Should be valid base64");
            // PNG magic bytes
            let bytes = decoded.unwrap();
            assert_eq!(&bytes[0..4], &[0x89, b'P', b'N', b'G']);
        }
        ResourceContents::TextResourceContents { .. } => panic!("Expected blob content"),
    }
}

#[test]
fn test_large_txt_resource() {
    let resource = get_large_txt_resource();
    assert_eq!(resource.uri, "test://static/large.txt");
}

#[test]
fn test_large_txt_content_is_over_10kb() {
    let content = get_large_txt_content();
    match content {
        ResourceContents::TextResourceContents { text, .. } => {
            assert!(text.len() > 10_000, "Large file should be > 10KB");
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

#[test]
fn test_read_static_resource() {
    let content = read_static_resource("test://static/hello.txt");
    assert!(content.is_some());

    let content = read_static_resource("test://nonexistent");
    assert!(content.is_none());
}

// UI App resource tests

#[test]
fn test_read_button_app_resource() {
    let content = read_static_resource("ui://button/app.html");
    assert!(content.is_some());
    match content.unwrap() {
        ResourceContents::TextResourceContents {
            text, mime_type, ..
        } => {
            assert_eq!(mime_type, Some("text/html;profile=mcp-app".to_string()));
            assert!(text.contains("McpApp"));
            assert!(text.contains("ui/initialize"));
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

#[test]
fn test_read_form_app_resource() {
    let content = read_static_resource("ui://form/app.html");
    assert!(content.is_some());
    match content.unwrap() {
        ResourceContents::TextResourceContents {
            text, mime_type, ..
        } => {
            assert_eq!(mime_type, Some("text/html;profile=mcp-app".to_string()));
            assert!(text.contains("McpApp"));
            assert!(text.contains("concat"));
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

#[test]
fn test_read_carousel_app_resource() {
    let content = read_static_resource("ui://carousel/app.html");
    assert!(content.is_some());
    match content.unwrap() {
        ResourceContents::TextResourceContents {
            text, mime_type, ..
        } => {
            assert_eq!(mime_type, Some("text/html;profile=mcp-app".to_string()));
            assert!(text.contains("McpApp"));
            assert!(text.contains("Carousel"));
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

// Dynamic resource tests

#[test]
fn test_list_dynamic_resources() {
    let resources = list_dynamic_resources();
    assert_eq!(resources.len(), 3);
}

#[test]
fn test_counter_resource() {
    let resource = get_counter_resource();
    assert_eq!(resource.uri, "test://dynamic/counter");
    assert_eq!(resource.name, "counter");
}

#[test]
fn test_counter_state_increments() {
    let state = CounterState::new();
    assert_eq!(state.increment(), 1);
    assert_eq!(state.increment(), 2);
    assert_eq!(state.increment(), 3);
}

#[test]
fn test_counter_content() {
    let content = get_counter_content(42);
    match content {
        ResourceContents::TextResourceContents { text, .. } => {
            assert!(text.contains("42"));
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

#[test]
fn test_timestamp_resource() {
    let resource = get_timestamp_resource();
    assert_eq!(resource.uri, "test://dynamic/timestamp");
}

#[test]
fn test_timestamp_content_is_rfc3339() {
    let content = get_timestamp_content();
    match content {
        ResourceContents::TextResourceContents { text, .. } => {
            // Should contain a valid timestamp
            assert!(text.contains("Current time:"));
            // RFC3339 format includes T and Z or timezone
            assert!(text.contains('T'));
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

#[test]
fn test_random_resource() {
    let resource = get_random_resource();
    assert_eq!(resource.uri, "test://dynamic/random");
}

#[test]
fn test_random_content_changes() {
    let content1 = get_random_content();
    let content2 = get_random_content();

    let text1 = match content1 {
        ResourceContents::TextResourceContents { text, .. } => text,
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    };
    let text2 = match content2 {
        ResourceContents::TextResourceContents { text, .. } => text,
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    };

    // Very unlikely to get same random numbers twice
    assert_ne!(text1, text2);
}

// ResourceHandler tests

#[test]
fn test_resource_handler_list_resources() {
    let handler = ResourceHandler::new();
    let result = handler.list_resources(None).unwrap();

    // 7 static (4 original + 3 UI apps) + 3 dynamic = 10 resources
    assert_eq!(result.resources.len(), 10);
}

#[test]
fn test_resource_handler_list_templates() {
    let handler = ResourceHandler::new();
    let result = handler.list_resource_templates(None).unwrap();

    assert_eq!(result.resource_templates.len(), 1);
    assert_eq!(
        result.resource_templates[0].uri_template,
        "test://files/{path}"
    );
}

#[test]
fn test_resource_handler_read_static() {
    let handler = ResourceHandler::new();
    let request = ReadResourceRequestParams {
        uri: "test://static/hello.txt".to_string(),
        meta: None,
    };
    let result = handler.read_resource(&request).unwrap();

    assert_eq!(result.contents.len(), 1);
}

#[test]
fn test_resource_handler_read_dynamic_counter() {
    let handler = ResourceHandler::new();
    let request = ReadResourceRequestParams {
        uri: "test://dynamic/counter".to_string(),
        meta: None,
    };

    // Counter should increment on each read
    let result1 = handler.read_resource(&request).unwrap();
    let result2 = handler.read_resource(&request).unwrap();

    let text1 = match &result1.contents[0] {
        ResourceContents::TextResourceContents { text, .. } => text.clone(),
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text"),
    };
    let text2 = match &result2.contents[0] {
        ResourceContents::TextResourceContents { text, .. } => text.clone(),
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text"),
    };

    assert!(text1.contains('1'));
    assert!(text2.contains('2'));
}

#[test]
fn test_resource_handler_read_template() {
    let handler = ResourceHandler::new();
    let request = ReadResourceRequestParams {
        uri: "test://files/example.txt".to_string(),
        meta: None,
    };
    let result = handler.read_resource(&request).unwrap();

    match &result.contents[0] {
        ResourceContents::TextResourceContents { text, .. } => {
            assert!(text.contains("example.txt"));
        }
        ResourceContents::BlobResourceContents { .. } => panic!("Expected text content"),
    }
}

#[test]
fn test_resource_handler_read_unknown() {
    let handler = ResourceHandler::new();
    let request = ReadResourceRequestParams {
        uri: "test://nonexistent".to_string(),
        meta: None,
    };
    let result = handler.read_resource(&request);

    assert!(result.is_err());
}

#[test]
fn test_resource_handler_subscribe_random() {
    let handler = ResourceHandler::new();
    let request = SubscribeRequestParams {
        uri: "test://dynamic/random".to_string(),
        meta: None,
    };
    let result = handler.subscribe(&request);

    assert!(result.is_ok());
}

#[test]
fn test_resource_handler_subscribe_non_subscribable() {
    let handler = ResourceHandler::new();
    let request = SubscribeRequestParams {
        uri: "test://static/hello.txt".to_string(),
        meta: None,
    };
    let result = handler.subscribe(&request);

    assert!(result.is_err());
}
