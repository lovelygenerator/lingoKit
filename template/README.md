# {{project-name}}

[日本語](README.ja.md)

A Rust library with a CLI (clap) and an MCP server (rmcp), localized with [lingokit](https://github.com/lovelygenerator/lingokit).

## Layout

```
locales/<locale>/*.ftl   translations (copy a folder to add a language)
crates/core              the library; build.rs checks the FTL files and generates LOCALES and msg
crates/cli               clap (lingokit::clap::parse localizes help and errors)
crates/mcp               rmcp (lingokit::rmcp localizes descriptions and schemas)
```

## Workflow

1. Write functions and errors in `crates/core`. Give each error `#[ftl("err-xxx")]` and add the key to `locales/*/core.ftl` (a missing key is a compile error).
2. Write `crates/cli` / `crates/mcp` with English doc comments.
3. Run `cargo test` and add the keys it lists to `cli.ftl` / `mcp.ftl`.

The FTL files are checked with [ftl-check](https://github.com/lovelygenerator/ftl-check) on every build: missing translations and mismatched variables are errors, possibly untranslated texts are warnings. The configuration is `ftl-check.toml`; the `ftl-check` command gives the same result.

## Locale

`--lang` (CLI only) → `{{env_prefix}}_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG` → `en`

```sh
cargo run --bin {{project-name}} -- --lang ja count README.md
```

## MCP client configuration

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

Tool descriptions and schemas stay in English unless `"{{env_prefix}}_MCP_LOCALIZE_SCHEMA": "true"` is added. Results and errors are always localized.
