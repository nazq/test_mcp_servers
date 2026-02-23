//! Server icon assets.
//!
//! Contains the base64-encoded SVG icon used in the server's `Implementation` metadata.

/// Base64-encoded SVG data URI for the server icon (test tube / beaker).
///
/// This icon is used in the MCP `Implementation` struct to identify the server
/// in client UIs. SVG format ensures it renders crisp at any resolution.
pub const SERVER_ICON_SVG: &str = concat!(
    "data:image/svg+xml;base64,",
    "PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA2NCA2",
    "NCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjMzM3YWIzIiBzdHJva2Utd2lkdGg9IjMiIHN0cm9rZS1s",
    "aW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBhdGggZD0iTTI0IDhWMjZM",
    "MTIgNDZhNCA0IDAgMDA0IDRoMzJhNCA0IDAgMDA0LTRMNDAgMjZWOCIvPjxsaW5lIHgxPSIyMCIg",
    "eTE9IjgiIHgyPSI0NCIgeTI9IjgiLz48cGF0aCBkPSJNMjQgMjZjNCA0IDEyIDQgMTYgMCIgc3Ry",
    "b2tlPSIjNWI5YmQzIi8+PGNpcmNsZSBjeD0iMjIiIGN5PSI0MCIgcj0iMyIgZmlsbD0iIzViOWJk",
    "MyIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMzQiIGN5PSIzNiIgcj0iMiIgZmlsbD0iIzMz",
    "N2FiMyIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iNDIiIGN5PSI0MiIgcj0iMi41IiBmaWxs",
    "PSIjNWI5YmQzIiBzdHJva2U9Im5vbmUiLz48L3N2Zz4=",
);
