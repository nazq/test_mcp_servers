//! Prompt templates: greeting, `code_review`, summarize, translate, `with_resource`.

use rmcp::{
    ErrorData as McpError,
    model::{Prompt, PromptArgument, PromptMessage, PromptMessageRole},
};
use std::{collections::HashMap, hash::BuildHasher};

/// Get all available prompts with their metadata.
#[must_use]
pub fn get_all_prompts() -> Vec<Prompt> {
    vec![
        Prompt::new(
            "greeting",
            Some("A simple greeting prompt"),
            Some(vec![
                PromptArgument::new("name")
                    .with_description("Name to greet")
                    .with_required(true),
            ]),
        ),
        Prompt::new(
            "code_review",
            Some("Multi-message prompt for code review"),
            Some(vec![
                PromptArgument::new("code")
                    .with_description("Code to review")
                    .with_required(true),
                PromptArgument::new("language")
                    .with_description("Programming language")
                    .with_required(true),
            ]),
        ),
        Prompt::new(
            "summarize",
            Some("Prompt to summarize text"),
            Some(vec![
                PromptArgument::new("text")
                    .with_description("Text to summarize")
                    .with_required(true),
            ]),
        ),
        Prompt::new(
            "translate",
            Some("Translate text to another language"),
            Some(vec![
                PromptArgument::new("text")
                    .with_description("Text to translate")
                    .with_required(true),
                PromptArgument::new("language")
                    .with_description("Target language")
                    .with_required(true),
            ]),
        ),
        Prompt::new(
            "with_resource",
            Some("Prompt that references an embedded resource"),
            Some(vec![]),
        ),
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

    Ok(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        format!("Hello, {name}!"),
    )])
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
        PromptMessage::new_text(
            PromptMessageRole::User,
            format!("Please review this {language} code:\n\n```{language}\n{code}\n```"),
        ),
        PromptMessage::new_text(
            PromptMessageRole::Assistant,
            "I'll review this code for quality, security, and best practices.",
        ),
    ])
}

fn generate_summarize<S: BuildHasher>(
    args: &HashMap<String, String, S>,
) -> Result<Vec<PromptMessage>, McpError> {
    let text = args
        .get("text")
        .ok_or_else(|| McpError::invalid_params("Missing required argument: text", None))?;

    Ok(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        format!("Please summarize the following text:\n\n{text}"),
    )])
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

    Ok(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        format!("Please translate the following text to {language}:\n\n{text}"),
    )])
}

fn generate_with_resource() -> Vec<PromptMessage> {
    vec![
        PromptMessage::new_text(
            PromptMessageRole::User,
            "Please analyze the resource at test://static/config",
        ),
        PromptMessage::new_text(
            PromptMessageRole::Assistant,
            "I'll analyze the configuration resource for you.",
        ),
    ]
}
