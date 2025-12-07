//! Prompt implementations for the MCP test server.
//!
//! Note: The `ServerHandler` trait from rmcp requires `Result` return types for all methods,
//! even when the implementation cannot fail. This is a trait requirement, not a design choice.

// ServerHandler trait requires Result return type even when implementation cannot fail
#![allow(clippy::unnecessary_wraps)]

pub mod templates;

use crate::server::McpTestServer;
use rmcp::{
    ErrorData as McpError,
    model::{GetPromptRequestParam, GetPromptResult, ListPromptsResult, PromptMessage},
    service::{RequestContext, RoleServer},
};
use std::collections::HashMap;

/// Convert JSON arguments map to a `HashMap<String, String>`.
///
/// This handles converting JSON values to strings, stripping quotes if necessary.
#[must_use]
pub fn convert_json_args(
    args: Option<rmcp::serde_json::Map<String, rmcp::serde_json::Value>>,
) -> HashMap<String, String> {
    args.map(|args| {
        args.into_iter()
            .map(|(k, v)| {
                let value = v
                    .as_str()
                    .map_or_else(|| v.to_string().trim_matches('"').to_string(), String::from);
                (k, value)
            })
            .collect()
    })
    .unwrap_or_default()
}

/// Get a prompt by name with the given arguments.
///
/// # Errors
///
/// Returns an error if the prompt is not found or if required arguments are missing.
pub fn get_prompt_by_name<S: std::hash::BuildHasher>(
    name: &str,
    arguments: &HashMap<String, String, S>,
) -> Result<(Vec<PromptMessage>, Option<String>), McpError> {
    // Generate prompt messages
    let messages = templates::generate_prompt(name, arguments)?;

    // Find prompt metadata for description
    let prompt = templates::get_all_prompts()
        .into_iter()
        .find(|p| p.name == name);

    Ok((messages, prompt.and_then(|p| p.description)))
}

impl McpTestServer {
    /// List all available prompts.
    ///
    /// Note: `&self` is required by the `ServerHandler` trait interface, even though
    /// this method doesn't use instance state. Returns `Result` for MCP protocol consistency.
    pub(crate) fn list_prompts_impl(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        let _ = self; // Required by ServerHandler trait
        let prompts = templates::get_all_prompts();

        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
        })
    }

    /// Get a specific prompt with substituted arguments.
    ///
    /// Note: `&self` is required by the `ServerHandler` trait interface, even though
    /// this method doesn't use instance state.
    pub(crate) fn get_prompt_impl(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let _ = self; // Required by ServerHandler trait
        let arguments = convert_json_args(request.arguments);
        let (messages, description) = get_prompt_by_name(&request.name, &arguments)?;

        Ok(GetPromptResult {
            description,
            messages,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_json_args_empty() {
        let result = convert_json_args(None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_convert_json_args_string_value() {
        let mut map = rmcp::serde_json::Map::new();
        map.insert("name".to_string(), rmcp::serde_json::json!("Alice"));
        let result = convert_json_args(Some(map));
        assert_eq!(result.get("name"), Some(&"Alice".to_string()));
    }

    #[test]
    fn test_convert_json_args_number_value() {
        let mut map = rmcp::serde_json::Map::new();
        map.insert("count".to_string(), rmcp::serde_json::json!(42));
        let result = convert_json_args(Some(map));
        assert_eq!(result.get("count"), Some(&"42".to_string()));
    }

    #[test]
    fn test_get_prompt_by_name_greeting() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), "Test".to_string());
        let (messages, description) = get_prompt_by_name("greeting", &args).unwrap();
        assert!(!messages.is_empty());
        assert!(description.is_some());
    }

    #[test]
    fn test_get_prompt_by_name_code_review() {
        let mut args = HashMap::new();
        args.insert("code".to_string(), "fn main() {}".to_string());
        args.insert("language".to_string(), "rust".to_string());
        let (messages, description) = get_prompt_by_name("code_review", &args).unwrap();
        assert!(!messages.is_empty());
        assert!(description.is_some());
    }

    #[test]
    fn test_get_prompt_by_name_unknown() {
        let args = HashMap::new();
        let result = get_prompt_by_name("nonexistent", &args);
        assert!(result.is_err());
    }
}
