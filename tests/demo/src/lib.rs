use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use lingokit::LocalizedError;
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData, ServerHandler, schemars, tool, tool_handler, tool_router};

lingokit::include_locales!();

#[derive(Debug, thiserror::Error, LocalizedError)]
pub enum Error {
    #[error("not found: {}", path.display())]
    #[ftl("err-not-found")]
    NotFound { path: PathBuf },
    #[error("too many: {count}")]
    #[ftl("err-too-many")]
    TooMany { count: usize, #[ftl(skip)] _internal: u8 },
    #[error("io")]
    #[ftl("err-io")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error, LocalizedError)]
#[error("empty")]
#[ftl("err-empty")]
pub struct EmptyError;

#[derive(Parser, Debug)]
#[command(name = "demo", version, about = "Demo CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Count words
    Count {
        /// Path
        path: PathBuf,
        /// Format
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Format {
    /// Text
    Text,
    /// Json
    Json,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CountParams {
    /// Path
    pub path: String,
}

pub struct Server {
    loc: lingokit::Localizer,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Server {
    pub fn new(loc: lingokit::Localizer) -> Self {
        let tool_router = lingokit::rmcp::localize_if_enabled(Self::tool_router(), &loc);
        Self { loc, tool_router }
    }
    pub fn base_router() -> ToolRouter<Self> {
        Self::tool_router()
    }
    /// Count words
    #[tool(name = "count_words")]
    async fn count_words(&self, Parameters(p): Parameters<CountParams>) -> Result<CallToolResult, ErrorData> {
        let e = Error::NotFound { path: p.path.into() };
        Ok(CallToolResult::error(vec![ContentBlock::text(self.loc.error(&e))]))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {}
