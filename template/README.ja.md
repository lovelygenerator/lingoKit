# {{project-name}}

[English](README.md)

Rust ライブラリ + CLI(clap)+ MCP サーバー(rmcp)。多言語対応には [lingokit](https://github.com/lovelygenerator/lingokit) を使います。

## 構成

```
locales/<lang>/*.ftl   翻訳(言語を追加するにはフォルダをコピーする)
crates/core            ライブラリ本体。build.rs で FTL を検証し、LOCALES と msg を生成する
crates/cli             clap(lingokit::clap::parse で help・エラーを翻訳)
crates/mcp             rmcp(lingokit::rmcp で説明・スキーマを翻訳)
```

## 開発の流れ

1. `crates/core` に関数とエラーを書く。エラーには `#[ftl("err-xxx")]` を付け、`locales/*/core.ftl` にキーを足す(キーがないとコンパイルエラー)
2. `crates/cli` / `crates/mcp` を英語の doc コメントで書く
3. `cargo test` を実行し、足りないキーの一覧を `cli.ftl` / `mcp.ftl` に追加する

FTL はビルドのたびに [ftl-check](https://github.com/lovelygenerator/ftl-check) で検査されます。翻訳の欠落や引数の不一致はビルドエラー、未翻訳の疑いなどは警告です。設定は `ftl-check.toml` にあり、CLI の `ftl-check` でも同じ結果になります。

## 言語の決まり方

`--lang`(CLI のみ)→ `{{env_prefix}}_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG` → `en`

```sh
cargo run --bin {{project-name}} -- --lang ja count README.md
```

## MCP クライアントの設定例

```json
{
  "mcpServers": {
    "{{project-name}}": {
      "command": "{{project-name}}-mcp",
      "env": { "{{env_prefix}}_LANG": "ja" }
    }
  }
}
```

ツールの説明・スキーマは既定で英語のままです。翻訳するには `"{{env_prefix}}_MCP_LOCALIZE_SCHEMA": "true"` を追加してください。
