use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // MCP にはロケール通知がないので環境変数だけで言語を決める
    let loc = lingokit::Localizer::from_env(&core_lib::LOCALES, None);
    mcp::Server::new(loc).serve(stdio()).await?.waiting().await?;
    Ok(())
}
