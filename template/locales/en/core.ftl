# コアライブラリのエラー(err-<kind>)
err-not-found = File not found: { $path }
err-empty = File is empty: { $path }
err-io = Failed to read { $path }

# 結果メッセージ(CLI / MCP 共通)
result-word-count = { $count ->
    [one] { $path } contains { $count } word.
   *[other] { $path } contains { $count } words.
}
