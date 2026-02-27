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

/// Parameters for the `ui_internal_only` tool (no input needed).
///
/// This tool has `visibility: "app"` — it is only callable from the MCP App
/// iframe, not visible to the LLM. Used to test client-side tool filtering.
#[derive(Debug, Deserialize)]
pub struct UiInternalOnlyParams {}
super::empty_params_schema!(
    UiInternalOnlyParams,
    "Parameters for the `ui_internal_only` tool (no input needed)."
);

/// Parameters for the `ui_resource_dashboard` tool (no input needed).
#[derive(Debug, Deserialize)]
pub struct UiResourceDashboardParams {}
super::empty_params_schema!(
    UiResourceDashboardParams,
    "Parameters for the `ui_resource_dashboard` tool (no input needed)."
);

/// Parameters for the `ui_resource_data_table` tool (no input needed).
#[derive(Debug, Deserialize)]
pub struct UiResourceDataTableParams {}
super::empty_params_schema!(
    UiResourceDataTableParams,
    "Parameters for the `ui_resource_data_table` tool (no input needed)."
);

/// Parameters for the `ui_resource_pipeline` tool (no input needed).
#[derive(Debug, Deserialize)]
pub struct UiResourcePipelineParams {}
super::empty_params_schema!(
    UiResourcePipelineParams,
    "Parameters for the `ui_resource_pipeline` tool (no input needed)."
);
