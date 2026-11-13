use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // MCP has no way to send a locale, so it comes from the environment
    let loc = lingokit::Localizer::from_env(&core_lib::LOCALES, None);
    mcp::Server::new(loc).serve(stdio()).await?.waiting().await?;
    Ok(())
}
