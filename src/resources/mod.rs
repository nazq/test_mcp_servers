//! Resource implementations for the MCP test server.

use std::sync::Arc;

use rmcp::{
    ErrorData,
    model::{
        AnnotateAble, ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParam,
        RawResourceTemplate, ReadResourceRequestParam, ReadResourceResult, SubscribeRequestParam,
        UnsubscribeRequestParam,
    },
};

pub mod dynamic_resources;
pub mod static_resources;

use dynamic_resources::CounterState;

/// Resource handler implementation.
///
/// This struct provides methods to handle MCP resource requests.
#[derive(Debug, Clone)]
pub struct ResourceHandler {
    counter_state: Arc<CounterState>,
}

impl ResourceHandler {
    /// Create a new resource handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter_state: Arc::new(CounterState::new()),
        }
    }

    /// List all available resources.
    ///
    /// # Errors
    ///
    /// This function currently does not return errors, but returns `Result`
    /// for API consistency with the MCP protocol.
    pub fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let mut resources = Vec::new();

        // Add static resources
        resources.extend(static_resources::list_static_resources());

        // Add dynamic resources
        resources.extend(dynamic_resources::list_dynamic_resources());

        // Add template resource
        // Note: The template itself is not listed as a resource, only via list_resource_templates

        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
        })
    }

    /// List all available resource templates.
    ///
    /// # Errors
    ///
    /// This function currently does not return errors, but returns `Result`
    /// for API consistency with the MCP protocol.
    pub fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        let template = RawResourceTemplate {
            uri_template: "test://files/{path}".to_string(),
            name: "files".to_string(),
            title: Some("File Template".to_string()),
            description: Some("Access files by path using a parameterized URI".to_string()),
            mime_type: Some("text/plain".to_string()),
        }
        .no_annotation();

        Ok(ListResourceTemplatesResult {
            resource_templates: vec![template],
            next_cursor: None,
        })
    }

    /// Read a resource by URI.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource URI is unknown or invalid.
    pub fn read_resource(
        &self,
        request: &ReadResourceRequestParam,
    ) -> Result<ReadResourceResult, ErrorData> {
        let uri = &request.uri;

        // Try static resources first
        if let Some(content) = static_resources::read_static_resource(uri) {
            return Ok(ReadResourceResult {
                contents: vec![content],
            });
        }

        // Try dynamic resources
        match uri.as_str() {
            "test://dynamic/counter" => {
                let value = self.counter_state.increment();
                let content = dynamic_resources::get_counter_content(value);
                return Ok(ReadResourceResult {
                    contents: vec![content],
                });
            }
            "test://dynamic/timestamp" => {
                let content = dynamic_resources::get_timestamp_content();
                return Ok(ReadResourceResult {
                    contents: vec![content],
                });
            }
            "test://dynamic/random" => {
                let content = dynamic_resources::get_random_content();
                return Ok(ReadResourceResult {
                    contents: vec![content],
                });
            }
            _ => {}
        }

        // Try template resource: test://files/{path}
        if let Some(path) = uri.strip_prefix("test://files/") {
            let content = rmcp::model::ResourceContents::TextResourceContents {
                uri: uri.clone(),
                mime_type: Some("text/plain".to_string()),
                text: format!("File content for path: {path}"),
                meta: None,
            };
            return Ok(ReadResourceResult {
                contents: vec![content],
            });
        }

        // Unknown resource
        Err(ErrorData::invalid_request(
            format!("Unknown resource URI: {uri}"),
            None,
        ))
    }

    /// Subscribe to resource updates.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource does not support subscriptions.
    pub fn subscribe(&self, request: &SubscribeRequestParam) -> Result<(), ErrorData> {
        // For now, we accept subscriptions to the random resource
        // In a real implementation, we would track subscriptions and send notifications
        let uri = &request.uri;

        match uri.as_str() {
            "test://dynamic/random" => {
                // Subscription accepted
                Ok(())
            }
            _ => Err(ErrorData::invalid_request(
                format!("Resource does not support subscriptions: {uri}"),
                None,
            )),
        }
    }

    /// Unsubscribe from resource updates.
    ///
    /// # Errors
    ///
    /// This function currently does not return errors, but returns `Result`
    /// for API consistency with the MCP protocol.
    pub const fn unsubscribe(&self, _request: &UnsubscribeRequestParam) -> Result<(), ErrorData> {
        // For now, we accept unsubscribe for any URI
        // In a real implementation, we would remove the subscription
        Ok(())
    }
}

impl Default for ResourceHandler {
    fn default() -> Self {
        Self::new()
    }
}
