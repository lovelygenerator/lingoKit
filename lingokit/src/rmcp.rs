//! Localizing [rmcp](https://docs.rs/rmcp) MCP servers (feature `rmcp`).
//!
//! | Target | Key |
//! |---|---|
//! | Tool description | `mcp-tool-<name>` |
//! | Tool title | `mcp-tool-<name>.title` (attribute) |
//! | Parameter description | `mcp-tool-<name>-param-<prop>` |
//! | Server instructions | `mcp-instructions` |
//!
//! Notes:
//! - This module uses the public `ToolRouter::map` of rmcp, so the rmcp version is pinned to `~3.10`.
//! - Only the top-level `properties` of an input schema are localized, not nested `$defs`.

use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::Tool;
use serde_json::Value;

use crate::{Locales, Localizer};

/// Whether tool descriptions and schemas are localized: only when
/// `<PREFIX>_MCP_LOCALIZE_SCHEMA` is `true`, `1`, `yes` or `on`. Off by default.
pub fn schema_localization_enabled(locales: &Locales) -> bool {
    std::env::var(format!("{}_MCP_LOCALIZE_SCHEMA", locales.env_prefix))
        .is_ok_and(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
}

/// Localizes the descriptions, titles and parameter descriptions of all tools.
pub fn localize<S>(mut router: ToolRouter<S>, loc: &Localizer) -> ToolRouter<S> {
    for route in router.map.values_mut() {
        localize_tool(&mut route.attr, loc);
    }
    router
}

/// [`localize`]s only when [`schema_localization_enabled`] is true.
pub fn localize_if_enabled<S>(router: ToolRouter<S>, loc: &Localizer) -> ToolRouter<S> {
    if schema_localization_enabled(loc.locales()) { localize(router, loc) } else { router }
}

/// The server instructions: `mcp-instructions` when schema localization is enabled, otherwise `default`.
pub fn instructions(loc: &Localizer, default: &str) -> String {
    if schema_localization_enabled(loc.locales()) {
        loc.get("mcp-instructions", None).unwrap_or_else(|| default.to_string())
    } else {
        default.to_string()
    }
}

/// Localizes one tool.
pub fn localize_tool(tool: &mut Tool, loc: &Localizer) {
    let key = format!("mcp-tool-{}", tool.name);
    if let Some(d) = loc.get(&key, None) {
        tool.description = Some(d.into());
    }
    if let Some(t) = loc.get(&format!("{key}.title"), None) {
        if let Some(a) = tool.annotations.as_mut() {
            a.title = Some(t.clone());
        }
        tool.title = Some(t);
    }
    let mut schema = (*tool.input_schema).clone();
    if let Some(Value::Object(props)) = schema.get_mut("properties") {
        for (prop, v) in props.iter_mut() {
            if let (Some(d), Value::Object(o)) = (loc.get(&format!("{key}-param-{prop}"), None), v) {
                o.insert("description".into(), Value::String(d));
            }
        }
    }
    tool.input_schema = Arc::new(schema);
}

/// The keys the tools of `router` need.
pub fn required_keys<S>(router: &ToolRouter<S>) -> Vec<String> {
    let mut keys = Vec::new();
    for route in router.map.values() {
        let key = format!("mcp-tool-{}", route.attr.name);
        if let Some(Value::Object(props)) = route.attr.input_schema.get("properties") {
            keys.extend(props.keys().map(|p| format!("{key}-param-{p}")));
        }
        keys.push(key);
    }
    keys.sort();
    keys
}

/// Asserts that the default locale has a key for every tool and parameter. Intended for tests.
///
/// # Panics
///
/// When keys are missing, with the list of keys to add.
pub fn assert_keys<S>(router: &ToolRouter<S>, locales: &'static Locales) {
    let required: Vec<Vec<String>> = required_keys(router).into_iter().map(|k| vec![k]).collect();
    crate::testing::assert_required_keys(locales, &required);
}
