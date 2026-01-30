//! UI Resource tool parameter structures.
//!
//! These tools return embedded HTML resources with `ui://` URIs to test
//! how MCP clients render and handle interactive UI content.

use serde::Deserialize;

/// Parameters for the `ui_resource_button` tool (no input needed).
#[derive(Debug, Deserialize)]
pub struct UiResourceButtonParams {}
super::empty_params_schema!(
    UiResourceButtonParams,
    "Parameters for the `ui_resource_button` tool (no input needed)."
);

/// Parameters for the `ui_resource_form` tool (no input needed).
#[derive(Debug, Deserialize)]
pub struct UiResourceFormParams {}
super::empty_params_schema!(
    UiResourceFormParams,
    "Parameters for the `ui_resource_form` tool (no input needed)."
);

/// Parameters for the `ui_resource_carousel` tool (no input needed).
#[derive(Debug, Deserialize)]
pub struct UiResourceCarouselParams {}
super::empty_params_schema!(
    UiResourceCarouselParams,
    "Parameters for the `ui_resource_carousel` tool (no input needed)."
);
