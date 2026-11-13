//! rmcp の MCP サーバーを FTL のキー規約に従って翻訳する(feature `rmcp`)。
//!
//! | 対象 | キー |
//! |---|---|
//! | ツールの説明 | `mcp-tool-<name>` |
//! | ツールのタイトル | `mcp-tool-<name>.title`(属性) |
//! | パラメータの説明 | `mcp-tool-<name>-param-<prop>` |
//! | サーバーの instructions | `mcp-instructions` |
//!
//! 注意:
//! - `ToolRouter.map`(pub)に依存している
//! - スキーマはトップレベルの `properties` だけを対象にする(`$defs` の入れ子は対象外)

use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::Tool;
use serde_json::Value;

use crate::{Locales, Localizer};

/// ツールの説明・スキーマを翻訳するか(既定: しない)。
///
/// 既定では翻訳せず、`<PREFIX>_MCP_LOCALIZE_SCHEMA=true`(`1` / `yes` / `on` も可)のときだけ翻訳する。
pub fn schema_localization_enabled(locales: &Locales) -> bool {
    std::env::var(format!("{}_MCP_LOCALIZE_SCHEMA", locales.env_prefix))
        .is_ok_and(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
}

/// 説明・タイトル・パラメータ説明を翻訳する。
pub fn localize<S>(mut router: ToolRouter<S>, loc: &Localizer) -> ToolRouter<S> {
    for route in router.map.values_mut() {
        localize_tool(&mut route.attr, loc);
    }
    router
}

/// [`schema_localization_enabled`] が true のときだけ [`localize`] する。
pub fn localize_if_enabled<S>(router: ToolRouter<S>, loc: &Localizer) -> ToolRouter<S> {
    if schema_localization_enabled(loc.locales()) { localize(router, loc) } else { router }
}

/// サーバーの instructions。スキーマ翻訳が有効なら `mcp-instructions`、無効なら `default` を返す。
pub fn instructions(loc: &Localizer, default: &str) -> String {
    if schema_localization_enabled(loc.locales()) {
        loc.get("mcp-instructions", None).unwrap_or_else(|| default.to_string())
    } else {
        default.to_string()
    }
}

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

/// 必須キーの一覧。
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

/// 全ツール・パラメータのキーが既定言語にあるかを検査する(テスト用)。
pub fn assert_keys<S>(router: &ToolRouter<S>, locales: &'static Locales) {
    let required: Vec<Vec<String>> = required_keys(router).into_iter().map(|k| vec![k]).collect();
    crate::testing::assert_required_keys(locales, &required);
}
