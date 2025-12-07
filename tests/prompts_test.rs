//! Integration tests for prompts implementation.

use std::collections::HashMap;

use mcp_test_server::prompts::templates::{generate_prompt, get_all_prompts};

#[test]
fn test_get_all_prompts_returns_five_prompts() {
    let prompts = get_all_prompts();
    assert_eq!(prompts.len(), 5);

    let names: Vec<&str> = prompts.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"greeting"));
    assert!(names.contains(&"code_review"));
    assert!(names.contains(&"summarize"));
    assert!(names.contains(&"translate"));
    assert!(names.contains(&"with_resource"));
}

#[test]
fn test_greeting_prompt() {
    let mut args = HashMap::new();
    args.insert("name".to_string(), "Alice".to_string());

    let result = generate_prompt("greeting", &args).unwrap();
    assert_eq!(result.len(), 1);

    match &result[0].content {
        rmcp::model::PromptMessageContent::Text { text } => {
            assert!(text.contains("Alice"));
        }
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_greeting_prompt_missing_name() {
    let args = HashMap::new();

    let result = generate_prompt("greeting", &args);
    assert!(result.is_err());
}

#[test]
fn test_code_review_prompt() {
    let mut args = HashMap::new();
    args.insert("code".to_string(), "fn main() {}".to_string());
    args.insert("language".to_string(), "rust".to_string());

    let result = generate_prompt("code_review", &args).unwrap();
    assert_eq!(result.len(), 2); // User message + Assistant response

    // First message should contain the code
    match &result[0].content {
        rmcp::model::PromptMessageContent::Text { text } => {
            assert!(text.contains("fn main()"));
            assert!(text.contains("rust"));
        }
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_code_review_missing_code() {
    let mut args = HashMap::new();
    args.insert("language".to_string(), "rust".to_string());

    let result = generate_prompt("code_review", &args);
    assert!(result.is_err());
}

#[test]
fn test_code_review_missing_language() {
    let mut args = HashMap::new();
    args.insert("code".to_string(), "fn main() {}".to_string());

    let result = generate_prompt("code_review", &args);
    assert!(result.is_err());
}

#[test]
fn test_summarize_prompt() {
    let mut args = HashMap::new();
    args.insert(
        "text".to_string(),
        "This is a long text to summarize.".to_string(),
    );

    let result = generate_prompt("summarize", &args).unwrap();
    assert_eq!(result.len(), 1);

    match &result[0].content {
        rmcp::model::PromptMessageContent::Text { text } => {
            assert!(text.contains("summarize"));
            assert!(text.contains("This is a long text"));
        }
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_translate_prompt() {
    let mut args = HashMap::new();
    args.insert("text".to_string(), "Hello world".to_string());
    args.insert("language".to_string(), "Spanish".to_string());

    let result = generate_prompt("translate", &args).unwrap();
    assert_eq!(result.len(), 1);

    match &result[0].content {
        rmcp::model::PromptMessageContent::Text { text } => {
            assert!(text.contains("Hello world"));
            assert!(text.contains("Spanish"));
        }
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_with_resource_prompt() {
    let args = HashMap::new();

    let result = generate_prompt("with_resource", &args).unwrap();
    assert_eq!(result.len(), 2); // User message + Assistant response

    match &result[0].content {
        rmcp::model::PromptMessageContent::Text { text } => {
            assert!(text.contains("test://static/config"));
        }
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_unknown_prompt() {
    let args = HashMap::new();

    let result = generate_prompt("nonexistent", &args);
    assert!(result.is_err());
}

#[test]
fn test_all_prompts_have_descriptions() {
    let prompts = get_all_prompts();

    for prompt in prompts {
        assert!(
            prompt.description.is_some(),
            "Prompt '{}' should have a description",
            prompt.name
        );
    }
}
