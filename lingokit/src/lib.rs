//! clap の CLI と rmcp の MCP サーバーを Fluent で多言語対応する薄いラッパー。
//!
//! # 使い方
//!
//! ```toml
//! [dependencies]
//! lingokit = { git = "...", features = ["clap", "rmcp"] }
//! [build-dependencies]
//! lingokit-build = { git = "..." }
//! ```
//!
//! ```ignore
//! // build.rs
//! fn main() { lingokit_build::Config::new("locales").generate(); }
//!
//! // lib.rs(クレートのルートに置く。`LOCALES` と `msg` が定義される)
//! lingokit::include_locales!();
//!
//! // CLI
//! let (cli, loc) = lingokit::clap::parse::<Cli>(&LOCALES);
//!
//! // MCP
//! let router = lingokit::rmcp::localize_if_enabled(Self::tool_router(), &loc);
//! ```

mod error;
pub mod locale;
mod localizer;
pub mod testing;

#[cfg(feature = "clap")]
pub mod clap;
#[cfg(feature = "rmcp")]
pub mod rmcp;

#[doc(hidden)]
pub mod __private;

pub use error::{LocalizedError, render_error_chain};
pub use fluent_bundle::{FluentArgs, FluentValue};
pub use localizer::Localizer;
pub use unic_langid::LanguageIdentifier;

#[cfg(feature = "derive")]
pub use lingokit_macros::LocalizedError;

/// build.rs(`lingokit-build`)が生成した翻訳リソース一式。
#[derive(Debug)]
pub struct Locales {
    /// 最終フォールバック言語
    pub default_lang: &'static str,
    /// 環境変数の接頭辞(`<PREFIX>_LANG` など)
    pub env_prefix: &'static str,
    /// 言語タグ → (ファイル名, FTL ソース)
    pub resources: &'static [(&'static str, &'static [(&'static str, &'static str)])],
    /// 既定言語のメッセージ ID(属性は `id.attr`)
    pub ids: &'static [&'static str],
}

/// メッセージ ID と引数の組。生成された `msg` の関数や derive が返す。
#[derive(Debug)]
pub struct Msg<'a> {
    pub id: &'static str,
    pub args: Option<FluentArgs<'a>>,
}

impl<'a> Msg<'a> {
    pub const fn new(id: &'static str) -> Self {
        Self { id, args: None }
    }

    pub fn with_args(id: &'static str, args: FluentArgs<'a>) -> Self {
        Self { id, args: Some(args) }
    }
}

/// build.rs が生成したコードを展開する。クレートのルート(lib.rs / main.rs)で呼ぶ。
///
/// `LOCALES: lingokit::Locales` と、型付きアクセサの `msg` モジュールが定義される。
#[macro_export]
macro_rules! include_locales {
    () => {
        include!(concat!(env!("OUT_DIR"), "/lingokit_locales.rs"));
    };
}
