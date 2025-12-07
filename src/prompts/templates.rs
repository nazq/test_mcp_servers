//! Prompt templates: greeting, `code_review`, summarize, translate, `with_resource`.

use rmcp::{
    ErrorData as McpError,
    model::{Prompt, PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole},
};
use std::{collections::HashMap, hash::BuildHasher};

/// Get all available prompts with their metadata.
#[must_use]
pub fn get_all_prompts() -> Vec<Prompt> {
    vec![
        Prompt {
            name: "greeting".to_string(),
            title: None,
            description: Some("A simple greeting prompt".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "name".to_string(),
                title: None,
                description: Some("Name to greet".to_string()),
                required: Some(true),
            }]),
            icons: None,
        },
        Prompt {
            name: "code_review".to_string(),
            title: None,
            description: Some("Multi-message prompt for code review".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "code".to_string(),
                    title: None,
                    description: Some("Code to review".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "language".to_string(),
                    title: None,
                    description: Some("Programming language".to_string()),
                    required: Some(true),
                },
            ]),
            icons: None,
        },
        Prompt {
            name: "summarize".to_string(),
            title: None,
            description: Some("Prompt to summarize text".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "text".to_string(),
                title: None,
                description: Some("Text to summarize".to_string()),
                required: Some(true),
            }]),
            icons: None,
        },
        Prompt {
            name: "translate".to_string(),
            title: None,
            description: Some("Translate text to another language".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "text".to_string(),
                    title: None,
                    description: Some("Text to translate".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "language".to_string(),
                    title: None,
                    description: Some("Target language".to_string()),
                    required: Some(true),
                },
            ]),
            icons: None,
        },
        Prompt {
            name: "with_resource".to_string(),
            title: None,
            description: Some("Prompt that references an embedded resource".to_string()),
            arguments: Some(vec![]),
            icons: None,
        },
    ]
}

/// Generate prompt messages for the given prompt name and arguments.
///
/// # Errors
///
/// Returns an error if:
/// - The prompt name is unknown
/// - Required arguments are missing
pub fn generate_prompt<S: BuildHasher>(
    name: &str,
    arguments: &HashMap<String, String, S>,
) -> Result<Vec<PromptMessage>, McpError> {
    match name {
        "greeting" => generate_greeting(arguments),
        "code_review" => generate_code_review(arguments),
        "summarize" => generate_summarize(arguments),
        "translate" => generate_translate(arguments),
        "with_resource" => Ok(generate_with_resource()),
        _ => Err(McpError::invalid_params(
            format!("Unknown prompt: {name}"),
            None,
        )),
    }
}

fn generate_greeting<S: BuildHasher>(
    args: &HashMap<String, String, S>,
) -> Result<Vec<PromptMessage>, McpError> {
    let name = args
        .get("name")
        .ok_or_else(|| McpError::invalid_params("Missing required argument: name", None))?;

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text {
            text: format!("Hello, {name}!"),
        },
    }])
}

fn generate_code_review<S: BuildHasher>(
    args: &HashMap<String, String, S>,
) -> Result<Vec<PromptMessage>, McpError> {
    let code = args
        .get("code")
        .ok_or_else(|| McpError::invalid_params("Missing required argument: code", None))?;
    let language = args
        .get("language")
        .ok_or_else(|| McpError::invalid_params("Missing required argument: language", None))?;

    Ok(vec![
        PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: format!("Please review this {language} code:\n\n```{language}\n{code}\n```"),
            },
        },
        PromptMessage {
            role: PromptMessageRole::Assistant,
            content: PromptMessageContent::Text {
                text: "I'll review this code for quality, security, and best practices."
                    .to_string(),
            },
        },
    ])
}

fn generate_summarize<S: BuildHasher>(
    args: &HashMap<String, String, S>,
) -> Result<Vec<PromptMessage>, McpError> {
    let text = args
        .get("text")
        .ok_or_else(|| McpError::invalid_params("Missing required argument: text", None))?;

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text {
            text: format!("Please summarize the following text:\n\n{text}"),
        },
    }])
}

fn generate_translate<S: BuildHasher>(
    args: &HashMap<String, String, S>,
) -> Result<Vec<PromptMessage>, McpError> {
    let text = args
        .get("text")
        .ok_or_else(|| McpError::invalid_params("Missing required argument: text", None))?;
    let language = args
        .get("language")
        .ok_or_else(|| McpError::invalid_params("Missing required argument: language", None))?;

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text {
            text: format!("Please translate the following text to {language}:\n\n{text}"),
        },
    }])
}

fn generate_with_resource() -> Vec<PromptMessage> {
    vec![
        PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text {
                text: "Please analyze the resource at test://static/config".to_string(),
            },
        },
        PromptMessage {
            role: PromptMessageRole::Assistant,
            content: PromptMessageContent::Text {
                text: "I'll analyze the configuration resource for you.".to_string(),
            },
        },
    ]
}
