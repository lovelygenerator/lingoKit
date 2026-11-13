//! The MCP server. Tool descriptions are written in English here and can be localized from the FTL files (mcp-*).

use std::path::Path;

use core_lib::msg;
use lingokit::Localizer;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData, ServerHandler, schemars, tool, tool_handler, tool_router};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CountWordsParams {
    /// Path to the text file
    pub path: String,
}

pub struct Server {
    loc: Localizer,
    tool_router: ToolRouter<Self>,
}

#[tool_router(vis = "pub")]
impl Server {
    pub fn new(loc: Localizer) -> Self {
        let tool_router = lingokit::rmcp::localize_if_enabled(Self::tool_router(), &loc);
        Self { loc, tool_router }
    }

    /// Count words in a text file
    #[tool(name = "count_words")]
    async fn count_words(&self, Parameters(p): Parameters<CountWordsParams>) -> Result<CallToolResult, ErrorData> {
        Ok(match core_lib::count_words(Path::new(&p.path)) {
            Ok(n) => CallToolResult::success(vec![ContentBlock::text(
                self.loc.msg(&msg::result_word_count(n, p.path.as_str())),
            )]),
            // Errors of the tool are returned as a tool result, so that the model can read them
            Err(e) => CallToolResult::error(vec![ContentBlock::text(self.loc.error(&e))]),
        })
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
            .with_instructions(lingokit::rmcp::instructions(
                &self.loc,
                "Sample MCP server. Use count_words to count words in a text file.",
            ))
    }
}
