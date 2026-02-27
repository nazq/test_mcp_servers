//! Core MCP server implementation.

use std::sync::Arc;

use axum::{Router, middleware, response::Json, routing::get};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use rand::Rng;
use rmcp::{
    handler::server::{ServerHandler, router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CompleteResult, CompletionInfo, ExtensionCapabilities, Icon, Implementation,
        ListResourceTemplatesResult, ListResourcesResult, ProtocolVersion, ReadResourceResult,
        Reference, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;

use crate::{
    auth::auth_middleware,
    config::Config,
    tools::{
        encoding::{
            Base64DecodeParams, Base64EncodeParams, HashSha256Params, JsonParseParams,
            JsonStringifyParams,
        },
        math::{AddParams, DivideParams, MultiplyParams, SubtractParams},
        string::{
            ConcatParams, EchoParams, LengthParams, LowercaseParams, ReverseParams, UppercaseParams,
        },
        testing::{
            BinaryDataParams, FailParams, FailWithMessageParams, LargeResponseParams,
            NestedDataParams, SleepParams, SlowEchoParams,
        },
        ui::{
            UiInternalOnlyParams, UiResourceButtonParams, UiResourceCarouselParams,
            UiResourceFormParams,
        },
        utility::{CurrentTimeParams, RandomNumberParams, RandomUuidParams},
    },
};

/// Build `_meta` for a UI tool linking it to its MCP App resource.
///
/// Sets the new format (`_meta.ui.resourceUri` + `_meta.ui.visibility`) and
/// legacy flat key (`_meta["ui/resourceUri"]`) for backward compatibility.
///
/// `visibility` controls where the tool appears:
/// - `"both"` — visible to both the LLM and the UI iframe
/// - `"app"` — only callable from the iframe, hidden from the LLM
fn ui_meta(resource_uri: &str, visibility: &str) -> rmcp::model::Meta {
    debug_assert!(
        visibility == "both" || visibility == "app",
        "Invalid UI visibility: {visibility}"
    );

    let mut meta = rmcp::model::Meta::new();
    meta.insert(
        "ui".to_string(),
        serde_json::json!({ "resourceUri": resource_uri, "visibility": visibility }),
    );
    // Legacy flat key — only carries resourceUri. Visibility is new and not
    // supported by hosts that still read the flat key format.
    tracing::warn!(
        resource_uri,
        "Emitting deprecated flat key `ui/resourceUri` in tool _meta — \
         migrate clients to `_meta.ui.resourceUri`"
    );
    meta.insert(
        "ui/resourceUri".to_string(),
        serde_json::json!(resource_uri),
    );
    meta
}

/// Helper function to create nested JSON data.
fn create_nested(depth: usize) -> serde_json::Value {
    if depth == 0 {
        serde_json::json!("leaf")
    } else {
        serde_json::json!({
            "level": depth,
            "nested": create_nested(depth - 1)
        })
    }
}

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}

/// Health check handler.
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

/// The main MCP test server.
///
/// This server provides a comprehensive set of tools, prompts, and resources
/// for testing MCP client implementations.
#[derive(Debug, Clone)]
pub struct McpTestServer {
    config: Config,
    tool_router: ToolRouter<Self>,
    resource_handler: crate::resources::ResourceHandler,
    log_level: std::sync::Arc<std::sync::atomic::AtomicU8>,
}

impl McpTestServer {
    /// Create a new MCP test server with the given configuration.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tool_router: Self::tool_router(),
            resource_handler: crate::resources::ResourceHandler::new(),
            // Default to Info level (1)
            log_level: std::sync::Arc::new(std::sync::atomic::AtomicU8::new(1)),
        }
    }

    /// Run the server, listening on the configured host and port.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to bind or encounters a runtime error.
    ///
    /// # Panics
    ///
    /// Panics if the Ctrl+C signal handler cannot be installed.
    #[allow(clippy::cognitive_complexity)]
    pub async fn run(&self) -> anyhow::Result<()> {
        let addr = std::net::SocketAddr::new(self.config.host, self.config.port);
        tracing::info!(%addr, "Starting MCP Test Server");

        // Create cancellation token for graceful shutdown
        let ct = CancellationToken::new();

        // Setup Streamable HTTP transport
        let session_manager = Arc::new(LocalSessionManager::default());
        let streamable_http_config = StreamableHttpServerConfig {
            sse_keep_alive: Some(std::time::Duration::from_secs(15)),
            sse_retry: Some(std::time::Duration::from_secs(3)),
            stateful_mode: false,
            cancellation_token: ct.clone(),
        };

        // Clone self for the service factory closure
        let server_clone = self.clone();
        let streamable_http_service = StreamableHttpService::new(
            move || Ok(server_clone.clone()),
            session_manager,
            streamable_http_config,
        );

        // Build protected routes with auth middleware
        let protected_routes = Router::new()
            .route(
                "/mcp",
                axum::routing::get_service(streamable_http_service.clone()),
            )
            .route(
                "/mcp",
                axum::routing::post_service(streamable_http_service.clone()),
            )
            .route(
                "/mcp",
                axum::routing::delete_service(streamable_http_service),
            )
            .layer(middleware::from_fn_with_state(
                self.config.clone(),
                auth_middleware,
            ));

        // Build the main router combining public and protected routes
        let app = Router::new()
            .route("/health", get(health_check))
            .merge(protected_routes)
            .layer(CorsLayer::permissive());

        // Bind TCP listener
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!(%addr, "Server listening on Streamable HTTP (/mcp) transport");

        // Setup graceful shutdown
        let shutdown_ct = ct.clone();
        let shutdown = async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl+C");
            tracing::info!("Shutdown signal received, draining connections...");
            shutdown_ct.cancel();
        };

        // Run the server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await?;

        tracing::info!("Server shutdown complete");
        Ok(())
    }

    /// Get the server configuration.
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }
}

/// Tool router implementation for aggregating tools.
#[tool_router]
impl McpTestServer {
    // Math tools

    /// Add two numbers together.
    #[tool(description = "Add two numbers together")]
    async fn add(&self, Parameters(params): Parameters<AddParams>) -> String {
        let result = params.a + params.b;
        result.to_string()
    }

    /// Subtract two numbers.
    #[tool(description = "Subtract second number from first number")]
    async fn subtract(&self, Parameters(params): Parameters<SubtractParams>) -> String {
        let result = params.a - params.b;
        result.to_string()
    }

    /// Multiply two numbers.
    #[tool(description = "Multiply two numbers together")]
    async fn multiply(&self, Parameters(params): Parameters<MultiplyParams>) -> String {
        let result = params.a * params.b;
        result.to_string()
    }

    /// Divide two numbers with zero check.
    #[tool(description = "Divide first number by second number")]
    async fn divide(&self, Parameters(params): Parameters<DivideParams>) -> Result<String, String> {
        if params.b == 0.0 {
            return Err("Division by zero".to_string());
        }
        let result = params.a / params.b;
        Ok(result.to_string())
    }

    // String tools

    /// Echo text back to the caller.
    #[tool(description = "Echo the input text back")]
    async fn echo(&self, Parameters(params): Parameters<EchoParams>) -> String {
        params.text
    }

    /// Concatenate multiple strings.
    #[tool(description = "Concatenate multiple strings together")]
    async fn concat(&self, Parameters(params): Parameters<ConcatParams>) -> String {
        params.strings.join("")
    }

    /// Convert text to uppercase.
    #[tool(description = "Convert text to uppercase")]
    async fn uppercase(&self, Parameters(params): Parameters<UppercaseParams>) -> String {
        params.text.to_uppercase()
    }

    /// Convert text to lowercase.
    #[tool(description = "Convert text to lowercase")]
    async fn lowercase(&self, Parameters(params): Parameters<LowercaseParams>) -> String {
        params.text.to_lowercase()
    }

    /// Reverse a string.
    #[tool(description = "Reverse a string")]
    async fn reverse(&self, Parameters(params): Parameters<ReverseParams>) -> String {
        params.text.chars().rev().collect()
    }

    /// Get the length of a string.
    #[tool(description = "Get the length of a string")]
    async fn length(&self, Parameters(params): Parameters<LengthParams>) -> String {
        params.text.len().to_string()
    }

    // Encoding tools

    /// Parse a JSON string into a value.
    #[tool(description = "Parse a JSON string")]
    async fn json_parse(
        &self,
        Parameters(params): Parameters<JsonParseParams>,
    ) -> Result<String, String> {
        let value: serde_json::Value =
            serde_json::from_str(&params.json).map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
    }

    /// Convert a value to a JSON string.
    #[tool(description = "Convert a value to JSON string")]
    async fn json_stringify(
        &self,
        Parameters(params): Parameters<JsonStringifyParams>,
    ) -> Result<String, String> {
        serde_json::to_string(&params.value).map_err(|e| e.to_string())
    }

    /// Base64 encode a string.
    #[tool(description = "Base64 encode a string")]
    async fn base64_encode(&self, Parameters(params): Parameters<Base64EncodeParams>) -> String {
        BASE64.encode(params.text.as_bytes())
    }

    /// Base64 decode a string.
    #[tool(description = "Base64 decode a string")]
    async fn base64_decode(
        &self,
        Parameters(params): Parameters<Base64DecodeParams>,
    ) -> Result<String, String> {
        let decoded = BASE64
            .decode(params.encoded.as_bytes())
            .map_err(|e| e.to_string())?;
        String::from_utf8(decoded).map_err(|e| e.to_string())
    }

    /// Generate SHA-256 hash of text.
    #[tool(description = "Generate SHA-256 hash of text")]
    async fn hash_sha256(&self, Parameters(params): Parameters<HashSha256Params>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(params.text.as_bytes());
        let result = hasher.finalize();
        format!("{result:x}")
    }

    // Utility tools

    /// Generate a random number in the specified range.
    #[tool(description = "Generate a random number in range [min, max]")]
    async fn random_number(
        &self,
        Parameters(params): Parameters<RandomNumberParams>,
    ) -> Result<String, String> {
        if params.min > params.max {
            return Err("min must be less than or equal to max".to_string());
        }
        let mut rng = rand::rng();
        let result = rng.random_range(params.min..=params.max);
        Ok(result.to_string())
    }

    /// Generate a random UUID v4.
    #[tool(description = "Generate a random UUID v4")]
    async fn random_uuid(&self, Parameters(_params): Parameters<RandomUuidParams>) -> String {
        let uuid = uuid::Uuid::new_v4();
        uuid.to_string()
    }

    /// Get the current UTC timestamp.
    #[tool(description = "Get the current UTC timestamp")]
    async fn current_time(&self, Parameters(_params): Parameters<CurrentTimeParams>) -> String {
        let now = Utc::now();
        now.to_rfc3339()
    }

    // Testing tools

    /// Sleep for a specified duration.
    #[tool(description = "Sleep for specified milliseconds")]
    async fn sleep(&self, Parameters(params): Parameters<SleepParams>) -> String {
        tokio::time::sleep(tokio::time::Duration::from_millis(params.duration_ms)).await;
        format!("Slept for {}ms", params.duration_ms)
    }

    /// Always returns an error.
    #[tool(description = "Always returns an error")]
    async fn fail(&self, Parameters(_params): Parameters<FailParams>) -> Result<String, String> {
        Err("This tool always fails".to_string())
    }

    /// Returns an error with a custom message.
    #[tool(description = "Returns an error with custom message")]
    async fn fail_with_message(
        &self,
        Parameters(params): Parameters<FailWithMessageParams>,
    ) -> Result<String, String> {
        Err(params.message)
    }

    /// Echo text after a delay.
    #[tool(description = "Echo text after specified delay")]
    async fn slow_echo(&self, Parameters(params): Parameters<SlowEchoParams>) -> String {
        tokio::time::sleep(tokio::time::Duration::from_millis(params.delay_ms)).await;
        params.text
    }

    /// Generate deeply nested JSON data.
    #[tool(description = "Generate deeply nested JSON data")]
    async fn nested_data(
        &self,
        Parameters(params): Parameters<NestedDataParams>,
    ) -> Result<String, String> {
        let data = create_nested(params.depth);
        serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
    }

    /// Generate a large text response.
    #[tool(description = "Generate a large text response")]
    async fn large_response(&self, Parameters(params): Parameters<LargeResponseParams>) -> String {
        let line = "This is a line of text to create a large response.
";
        let lines_needed = params.size_bytes.div_ceil(line.len());
        line.repeat(lines_needed)
    }

    /// Generate random binary data and return as base64.
    #[tool(description = "Generate random binary data as base64")]
    async fn binary_data(&self, Parameters(params): Parameters<BinaryDataParams>) -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        let data: Vec<u8> = (0..params.size_bytes).map(|_| rng.random()).collect();
        BASE64.encode(&data)
    }

    /// No-operation tool for testing tool invocation without side effects.
    #[tool(description = "No-op tool that returns immediately")]
    async fn noop(&self) -> String {
        "ok".to_string()
    }

    // MCP App tools
    //
    // These tools declare `_meta.ui.resourceUri` so MCP Apps-capable hosts
    // (VS Code Insiders, Claude Desktop) fetch and render the interactive HTML
    // from the corresponding `ui://` resource via `resources/read`.
    // The tool itself returns plain text — the host pushes it to the iframe
    // via `ui/notifications/tool-result`.

    /// Interactive button app — host renders `ui://button/app.html`.
    #[tool(
        description = "Returns a single interactive UI button (tests single UI resource rendering)",
        meta = ui_meta("ui://button/app.html", "both")
    )]
    async fn ui_resource_button(
        &self,
        Parameters(_params): Parameters<UiResourceButtonParams>,
    ) -> String {
        "Button UI ready. Click the button to call the echo tool.".to_string()
    }

    /// Interactive form app — host renders `ui://form/app.html`.
    #[tool(
        description = "Returns a single interactive UI form (tests single UI resource rendering)",
        meta = ui_meta("ui://form/app.html", "both")
    )]
    async fn ui_resource_form(
        &self,
        Parameters(_params): Parameters<UiResourceFormParams>,
    ) -> String {
        "Form UI ready. Fill in the form and submit to call the concat tool.".to_string()
    }

    /// Interactive carousel app — host renders `ui://carousel/app.html`.
    #[tool(
        description = "Returns 3 interactive UI cards (tests multi-resource carousel rendering)",
        meta = ui_meta("ui://carousel/app.html", "both")
    )]
    async fn ui_resource_carousel(
        &self,
        Parameters(_params): Parameters<UiResourceCarouselParams>,
    ) -> String {
        "Carousel UI ready. 3 interactive cards loaded. Click a card to call the echo tool."
            .to_string()
    }

    /// Internal-only UI tool — hidden from the LLM, only callable from the iframe.
    ///
    /// Tests client-side tool filtering based on `_meta.ui.visibility: "app"`.
    #[tool(
        description = "Internal tool only callable from a UI iframe (tests app-only visibility filtering)",
        meta = ui_meta("ui://internal_only/app.html", "app")
    )]
    async fn ui_internal_only(
        &self,
        Parameters(_params): Parameters<UiInternalOnlyParams>,
    ) -> String {
        serde_json::json!({
            "status": "ok",
            "message": "Internal-only tool called successfully",
            "visibility": "app"
        })
        .to_string()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for McpTestServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .enable_prompts()
                .enable_prompts_list_changed()
                .enable_resources()
                .enable_resources_list_changed()
                .enable_resources_subscribe()
                .enable_logging()
                .enable_completions()
                .enable_extensions_with({
                    let mut ext = ExtensionCapabilities::new();
                    ext.insert(
                        "io.modelcontextprotocol/ui".to_string(),
                        serde_json::Map::new(),
                    );
                    ext
                })
                .build(),
            server_info: Implementation {
                name: "mcp-test-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("MCP Test Server".to_string()),
                description: Some(
                    "Comprehensive MCP test server for validating client implementations."
                        .to_string(),
                ),
                icons: Some(vec![Icon {
                    src: crate::icons::SERVER_ICON_SVG.to_string(),
                    mime_type: Some("image/svg+xml".to_string()),
                    sizes: Some(vec!["any".to_string()]),
                }]),
                website_url: Some("https://github.com/nazq/test_mcp_servers".to_string()),
            },
            instructions: Some(
                "A comprehensive MCP test server providing tools, prompts, and resources \
                 for testing MCP client implementations."
                    .to_string(),
            ),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<rmcp::model::ListPromptsResult, rmcp::ErrorData> {
        self.list_prompts_impl(context)
    }

    async fn get_prompt(
        &self,
        request: rmcp::model::GetPromptRequestParams,
        context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<rmcp::model::GetPromptResult, rmcp::ErrorData> {
        self.get_prompt_impl(request, context)
    }

    async fn list_resources(
        &self,
        request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::ErrorData> {
        self.resource_handler.list_resources(request)
    }

    async fn list_resource_templates(
        &self,
        request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListResourceTemplatesResult, rmcp::ErrorData> {
        self.resource_handler.list_resource_templates(request)
    }

    async fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ReadResourceResult, rmcp::ErrorData> {
        self.resource_handler.read_resource(&request)
    }

    async fn subscribe(
        &self,
        request: rmcp::model::SubscribeRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<(), rmcp::ErrorData> {
        self.resource_handler.subscribe(&request)
    }

    async fn unsubscribe(
        &self,
        request: rmcp::model::UnsubscribeRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<(), rmcp::ErrorData> {
        self.resource_handler.unsubscribe(&request)
    }

    async fn complete(
        &self,
        request: rmcp::model::CompleteRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CompleteResult, rmcp::ErrorData> {
        // Provide completions based on the reference type and argument
        let values = match &request.r#ref {
            Reference::Prompt(prompt_ref) => {
                // For prompt arguments, provide completions based on the prompt name and argument
                match (prompt_ref.name.as_str(), request.argument.name.as_str()) {
                    ("greeting", "name") => {
                        vec![
                            "Alice".into(),
                            "Bob".into(),
                            "Charlie".into(),
                            "World".into(),
                        ]
                    }
                    ("code_review", "language") => {
                        vec![
                            "rust".into(),
                            "python".into(),
                            "javascript".into(),
                            "typescript".into(),
                            "go".into(),
                        ]
                    }
                    ("translate", "language") => {
                        vec![
                            "Spanish".into(),
                            "French".into(),
                            "German".into(),
                            "Japanese".into(),
                            "Chinese".into(),
                        ]
                    }
                    _ => vec![],
                }
            }
            Reference::Resource(resource_ref) => {
                // For resource URIs, provide some example paths
                if resource_ref.uri.starts_with("test://files/") {
                    vec![
                        "example.txt".into(),
                        "data.json".into(),
                        "config.yaml".into(),
                    ]
                } else {
                    vec![]
                }
            }
        };

        // Filter values based on the argument value prefix if provided
        let filtered: Vec<String> = if request.argument.value.is_empty() {
            values
        } else {
            let prefix = &request.argument.value;
            values
                .into_iter()
                .filter(|v| v.starts_with(prefix))
                .collect()
        };

        Ok(CompleteResult {
            completion: CompletionInfo {
                values: filtered,
                total: None,
                has_more: Some(false),
            },
        })
    }

    async fn set_level(
        &self,
        request: rmcp::model::SetLevelRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<(), rmcp::ErrorData> {
        use std::sync::atomic::Ordering;

        // Map LoggingLevel to u8 for atomic storage
        let level = match request.level {
            rmcp::model::LoggingLevel::Debug => 0,
            rmcp::model::LoggingLevel::Info => 1,
            rmcp::model::LoggingLevel::Notice => 2,
            rmcp::model::LoggingLevel::Warning => 3,
            rmcp::model::LoggingLevel::Error => 4,
            rmcp::model::LoggingLevel::Critical => 5,
            rmcp::model::LoggingLevel::Alert => 6,
            rmcp::model::LoggingLevel::Emergency => 7,
        };

        self.log_level.store(level, Ordering::SeqCst);
        tracing::info!("Log level set to {:?}", request.level);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test server
    fn test_server() -> McpTestServer {
        McpTestServer::new(Config::default())
    }

    // =============================================================================
    // MATH TOOL TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_add() {
        let server = test_server();
        let result = server.add(Parameters(AddParams { a: 1.5, b: 2.5 })).await;
        assert_eq!(result, "4");
    }

    #[tokio::test]
    async fn test_subtract() {
        let server = test_server();
        let result = server
            .subtract(Parameters(SubtractParams { a: 10.0, b: 3.0 }))
            .await;
        assert_eq!(result, "7");
    }

    #[tokio::test]
    async fn test_multiply() {
        let server = test_server();
        let result = server
            .multiply(Parameters(MultiplyParams { a: 4.0, b: 5.0 }))
            .await;
        assert_eq!(result, "20");
    }

    #[tokio::test]
    async fn test_divide() {
        let server = test_server();
        let result = server
            .divide(Parameters(DivideParams { a: 20.0, b: 4.0 }))
            .await
            .unwrap();
        assert_eq!(result, "5");
    }

    #[tokio::test]
    async fn test_divide_by_zero() {
        let server = test_server();
        let result = server
            .divide(Parameters(DivideParams { a: 10.0, b: 0.0 }))
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Division by zero");
    }

    // =============================================================================
    // STRING TOOL TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_echo() {
        let server = test_server();
        let result = server
            .echo(Parameters(EchoParams {
                text: "hello".to_string(),
            }))
            .await;
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_concat() {
        let server = test_server();
        let result = server
            .concat(Parameters(ConcatParams {
                strings: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            }))
            .await;
        assert_eq!(result, "abc");
    }

    #[tokio::test]
    async fn test_uppercase() {
        let server = test_server();
        let result = server
            .uppercase(Parameters(UppercaseParams {
                text: "hello".to_string(),
            }))
            .await;
        assert_eq!(result, "HELLO");
    }

    #[tokio::test]
    async fn test_lowercase() {
        let server = test_server();
        let result = server
            .lowercase(Parameters(LowercaseParams {
                text: "HELLO".to_string(),
            }))
            .await;
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_reverse() {
        let server = test_server();
        let result = server
            .reverse(Parameters(ReverseParams {
                text: "abc".to_string(),
            }))
            .await;
        assert_eq!(result, "cba");
    }

    #[tokio::test]
    async fn test_length() {
        let server = test_server();
        let result = server
            .length(Parameters(LengthParams {
                text: "hello".to_string(),
            }))
            .await;
        assert_eq!(result, "5");
    }

    // =============================================================================
    // ENCODING TOOL TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_json_parse() {
        let server = test_server();
        let result = server
            .json_parse(Parameters(JsonParseParams {
                json: r#"{"key": "value"}"#.to_string(),
            }))
            .await
            .unwrap();
        assert!(result.contains("key"));
        assert!(result.contains("value"));
    }

    #[tokio::test]
    async fn test_json_parse_invalid() {
        let server = test_server();
        let result = server
            .json_parse(Parameters(JsonParseParams {
                json: "not valid json".to_string(),
            }))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_json_stringify() {
        let server = test_server();
        let result = server
            .json_stringify(Parameters(JsonStringifyParams {
                value: serde_json::json!({"foo": "bar"}),
            }))
            .await
            .unwrap();
        assert!(result.contains("foo"));
        assert!(result.contains("bar"));
    }

    #[tokio::test]
    async fn test_base64_encode() {
        let server = test_server();
        let result = server
            .base64_encode(Parameters(Base64EncodeParams {
                text: "hello".to_string(),
            }))
            .await;
        assert_eq!(result, "aGVsbG8=");
    }

    #[tokio::test]
    async fn test_base64_decode() {
        let server = test_server();
        let result = server
            .base64_decode(Parameters(Base64DecodeParams {
                encoded: "aGVsbG8=".to_string(),
            }))
            .await
            .unwrap();
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_base64_decode_invalid() {
        let server = test_server();
        let result = server
            .base64_decode(Parameters(Base64DecodeParams {
                encoded: "not valid base64!!!".to_string(),
            }))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_hash_sha256() {
        let server = test_server();
        let result = server
            .hash_sha256(Parameters(HashSha256Params {
                text: "hello".to_string(),
            }))
            .await;
        // SHA256 of "hello"
        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    // =============================================================================
    // UTILITY TOOL TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_random_number() {
        let server = test_server();
        let result = server
            .random_number(Parameters(RandomNumberParams { min: 1, max: 10 }))
            .await
            .unwrap();
        let num: i64 = result.parse().unwrap();
        assert!((1..=10).contains(&num));
    }

    #[tokio::test]
    async fn test_random_number_invalid_range() {
        let server = test_server();
        let result = server
            .random_number(Parameters(RandomNumberParams { min: 10, max: 1 }))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_random_uuid() {
        let server = test_server();
        let result = server.random_uuid(Parameters(RandomUuidParams {})).await;
        // UUID format check
        assert_eq!(result.len(), 36);
        assert_eq!(result.chars().filter(|c| *c == '-').count(), 4);
    }

    #[tokio::test]
    async fn test_current_time() {
        let server = test_server();
        let result = server.current_time(Parameters(CurrentTimeParams {})).await;
        // Should be valid RFC3339
        assert!(result.contains('T'));
    }

    // =============================================================================
    // TESTING TOOL TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_sleep() {
        let server = test_server();
        let start = std::time::Instant::now();
        let result = server
            .sleep(Parameters(SleepParams { duration_ms: 50 }))
            .await;
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 50);
        assert!(result.contains("50"));
    }

    #[tokio::test]
    async fn test_fail() {
        let server = test_server();
        let result = server.fail(Parameters(FailParams {})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("always fails"));
    }

    #[tokio::test]
    async fn test_fail_with_message() {
        let server = test_server();
        let result = server
            .fail_with_message(Parameters(FailWithMessageParams {
                message: "custom error".to_string(),
            }))
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "custom error");
    }

    #[tokio::test]
    async fn test_slow_echo() {
        let server = test_server();
        let start = std::time::Instant::now();
        let result = server
            .slow_echo(Parameters(SlowEchoParams {
                text: "hello".to_string(),
                delay_ms: 50,
            }))
            .await;
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 50);
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_nested_data() {
        let server = test_server();
        let result = server
            .nested_data(Parameters(NestedDataParams { depth: 3 }))
            .await
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["level"], 3);
        assert_eq!(parsed["nested"]["level"], 2);
        assert_eq!(parsed["nested"]["nested"]["level"], 1);
    }

    #[tokio::test]
    async fn test_large_response() {
        let server = test_server();
        let result = server
            .large_response(Parameters(LargeResponseParams { size_bytes: 1000 }))
            .await;
        assert!(result.len() >= 1000);
    }

    #[tokio::test]
    async fn test_binary_data() {
        use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

        let server = test_server();
        let result = server
            .binary_data(Parameters(BinaryDataParams { size_bytes: 100 }))
            .await;
        // Should be valid base64
        let decoded = BASE64.decode(&result).unwrap();
        assert_eq!(decoded.len(), 100);
    }

    // =============================================================================
    // SERVER INFO TESTS
    // =============================================================================

    #[test]
    fn test_server_info() {
        let server = test_server();
        let info = server.get_info();

        assert_eq!(info.server_info.name, "mcp-test-server");
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.resources.is_some());
        assert!(info.capabilities.prompts.is_some());
        assert!(info.capabilities.logging.is_some());
        assert!(info.capabilities.completions.is_some());
    }

    #[test]
    fn test_server_advertises_ui_extension() {
        let server = test_server();
        let info = server.get_info();
        let extensions = info
            .capabilities
            .extensions
            .as_ref()
            .expect("extensions should be Some");
        assert!(
            extensions.contains_key("io.modelcontextprotocol/ui"),
            "should advertise io.modelcontextprotocol/ui extension"
        );
    }

    #[test]
    fn test_config_accessor() {
        let config = Config::default();
        let server = McpTestServer::new(config.clone());
        assert_eq!(server.config().port, config.port);
    }

    // =============================================================================
    // HELPER FUNCTION TESTS
    // =============================================================================

    #[test]
    fn test_create_nested() {
        let nested = create_nested(0);
        assert_eq!(nested, "leaf");

        let nested = create_nested(2);
        assert_eq!(nested["level"], 2);
        assert_eq!(nested["nested"]["level"], 1);
        assert_eq!(nested["nested"]["nested"], "leaf");
    }

    // =============================================================================
    // UI RESOURCE TOOL TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_ui_resource_button() {
        let server = test_server();
        let result = server
            .ui_resource_button(Parameters(UiResourceButtonParams {}))
            .await;
        assert!(result.contains("Button UI ready"));
    }

    #[tokio::test]
    async fn test_ui_resource_form() {
        let server = test_server();
        let result = server
            .ui_resource_form(Parameters(UiResourceFormParams {}))
            .await;
        assert!(result.contains("Form UI ready"));
    }

    #[tokio::test]
    async fn test_ui_resource_carousel() {
        let server = test_server();
        let result = server
            .ui_resource_carousel(Parameters(UiResourceCarouselParams {}))
            .await;
        assert!(result.contains("Carousel UI ready"));
    }

    #[tokio::test]
    async fn test_ui_internal_only() {
        let server = test_server();
        let result = server
            .ui_internal_only(Parameters(UiInternalOnlyParams {}))
            .await;
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["visibility"], "app");
    }
}
