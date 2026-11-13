# Errors of the library (err-<kind>)
err-not-found = File not found: { $path }
err-empty = File is empty: { $path }
err-io = Failed to read { $path }

# Results (shared by the CLI and the MCP server)
result-word-count = { $count ->
    [one] { $path } contains { $count } word.
   *[other] { $path } contains { $count } words.
}
