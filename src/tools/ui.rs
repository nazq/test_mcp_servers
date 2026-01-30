//! UI Resource tool parameter structures.
//!
//! These tools return embedded HTML resources with `ui://` URIs to test
//! how MCP clients render and handle interactive UI content.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for the `ui_resource_button` tool (no input needed).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UiResourceButtonParams {}

/// Parameters for the `ui_resource_form` tool (no input needed).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UiResourceFormParams {}

/// Parameters for the `ui_resource_carousel` tool (no input needed).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UiResourceCarouselParams {}
