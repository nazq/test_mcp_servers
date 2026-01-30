//! Static resources: hello.txt, data.json, image.png, large.txt.

use rmcp::model::{AnnotateAble, RawResource, Resource, ResourceContents};

/// Get the hello.txt static resource.
#[must_use]
pub fn get_hello_resource() -> Resource {
    RawResource {
        uri: "test://static/hello.txt".to_string(),
        name: "hello.txt".to_string(),
        title: None,
        description: Some("A simple hello world text file".to_string()),
        mime_type: Some("text/plain".to_string()),
        size: Some(13),
        icons: None,
    }
    .no_annotation()
}

/// Get the hello.txt content.
#[must_use]
pub fn get_hello_content() -> ResourceContents {
    ResourceContents::TextResourceContents {
        uri: "test://static/hello.txt".to_string(),
        mime_type: Some("text/plain".to_string()),
        text: "Hello, World!".to_string(),
        meta: None,
    }
}

/// Get the data.json static resource.
#[must_use]
pub fn get_data_json_resource() -> Resource {
    RawResource {
        uri: "test://static/data.json".to_string(),
        name: "data.json".to_string(),
        title: None,
        description: Some("Sample JSON data".to_string()),
        mime_type: Some("application/json".to_string()),
        size: Some(49),
        icons: None,
    }
    .no_annotation()
}

/// Get the data.json content.
#[must_use]
pub fn get_data_json_content() -> ResourceContents {
    ResourceContents::TextResourceContents {
        uri: "test://static/data.json".to_string(),
        mime_type: Some("application/json".to_string()),
        text: r#"{"name": "test", "version": "1.0", "items": [1, 2, 3]}"#.to_string(),
        meta: None,
    }
}

/// Get the image.png static resource.
#[must_use]
pub fn get_image_png_resource() -> Resource {
    RawResource {
        uri: "test://static/image.png".to_string(),
        name: "image.png".to_string(),
        title: None,
        description: Some("A 1x1 pixel PNG image".to_string()),
        mime_type: Some("image/png".to_string()),
        size: Some(68),
        icons: None,
    }
    .no_annotation()
}

/// Get the image.png content.
/// This is a base64-encoded 1x1 transparent PNG.
#[must_use]
pub fn get_image_png_content() -> ResourceContents {
    // 1x1 transparent PNG in base64
    let base64_png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    ResourceContents::BlobResourceContents {
        uri: "test://static/image.png".to_string(),
        mime_type: Some("image/png".to_string()),
        blob: base64_png.to_string(),
        meta: None,
    }
}

/// Get the large.txt static resource.
#[must_use]
pub fn get_large_txt_resource() -> Resource {
    RawResource {
        uri: "test://static/large.txt".to_string(),
        name: "large.txt".to_string(),
        title: None,
        description: Some("A large text file for pagination testing (10KB+)".to_string()),
        mime_type: Some("text/plain".to_string()),
        size: Some(11_100),
        icons: None,
    }
    .no_annotation()
}

/// Get the large.txt content.
/// Generates a text file larger than 10KB for pagination testing.
#[must_use]
pub fn get_large_txt_content() -> ResourceContents {
    use std::fmt::Write;

    // Generate ~10KB of text
    let mut text = String::with_capacity(11_000);

    for i in 0..150 {
        let _ = writeln!(
            text,
            "This is line number {i:05} of the large text file for pagination testing."
        );
    }

    ResourceContents::TextResourceContents {
        uri: "test://static/large.txt".to_string(),
        mime_type: Some("text/plain".to_string()),
        text,
        meta: None,
    }
}

/// MCP Apps MIME type for interactive UI resources.
const MCP_APP_MIME_TYPE: &str = "text/html;profile=mcp-app";

/// Get the button app UI resource.
#[must_use]
pub fn get_button_app_resource() -> Resource {
    RawResource {
        uri: "ui://button/app.html".to_string(),
        name: "button-app".to_string(),
        title: Some("Button App".to_string()),
        description: Some("Interactive button that calls the echo tool".to_string()),
        mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
        size: None,
        icons: None,
    }
    .no_annotation()
}

/// Get the button app HTML content with the MCP App shim inlined.
#[must_use]
pub fn get_button_app_content() -> ResourceContents {
    let shim = include_str!("../../ui_templates/mcp-app-shim.js");
    let html = include_str!("../../ui_templates/button.html").replace("// {{MCP_APP_SHIM}}", shim);
    ResourceContents::TextResourceContents {
        uri: "ui://button/app.html".to_string(),
        mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
        text: html,
        meta: None,
    }
}

/// Get the form app UI resource.
#[must_use]
pub fn get_form_app_resource() -> Resource {
    RawResource {
        uri: "ui://form/app.html".to_string(),
        name: "form-app".to_string(),
        title: Some("Form App".to_string()),
        description: Some("Interactive form that calls the concat tool".to_string()),
        mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
        size: None,
        icons: None,
    }
    .no_annotation()
}

/// Get the form app HTML content with the MCP App shim inlined.
#[must_use]
pub fn get_form_app_content() -> ResourceContents {
    let shim = include_str!("../../ui_templates/mcp-app-shim.js");
    let html = include_str!("../../ui_templates/form.html").replace("// {{MCP_APP_SHIM}}", shim);
    ResourceContents::TextResourceContents {
        uri: "ui://form/app.html".to_string(),
        mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
        text: html,
        meta: None,
    }
}

/// Get the carousel app UI resource.
#[must_use]
pub fn get_carousel_app_resource() -> Resource {
    RawResource {
        uri: "ui://carousel/app.html".to_string(),
        name: "carousel-app".to_string(),
        title: Some("Carousel App".to_string()),
        description: Some("Interactive carousel with 3 cards that call the echo tool".to_string()),
        mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
        size: None,
        icons: None,
    }
    .no_annotation()
}

/// Get the carousel app HTML content with the MCP App shim inlined.
#[must_use]
pub fn get_carousel_app_content() -> ResourceContents {
    let shim = include_str!("../../ui_templates/mcp-app-shim.js");
    let html = include_str!("../../ui_templates/card.html").replace("// {{MCP_APP_SHIM}}", shim);
    ResourceContents::TextResourceContents {
        uri: "ui://carousel/app.html".to_string(),
        mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
        text: html,
        meta: None,
    }
}

/// Get all static resources.
#[must_use]
pub fn list_static_resources() -> Vec<Resource> {
    vec![
        get_hello_resource(),
        get_data_json_resource(),
        get_image_png_resource(),
        get_large_txt_resource(),
        get_button_app_resource(),
        get_form_app_resource(),
        get_carousel_app_resource(),
    ]
}

/// Read a static resource by URI.
#[must_use]
pub fn read_static_resource(uri: &str) -> Option<ResourceContents> {
    match uri {
        "test://static/hello.txt" => Some(get_hello_content()),
        "test://static/data.json" => Some(get_data_json_content()),
        "test://static/image.png" => Some(get_image_png_content()),
        "test://static/large.txt" => Some(get_large_txt_content()),
        "ui://button/app.html" => Some(get_button_app_content()),
        "ui://form/app.html" => Some(get_form_app_content()),
        "ui://carousel/app.html" => Some(get_carousel_app_content()),
        _ => None,
    }
}
