#[test]
fn mcp_keys() {
    lingokit::rmcp::assert_keys(&mcp::Server::tool_router(), &core_lib::LOCALES);
}
