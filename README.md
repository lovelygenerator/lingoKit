# lingokit

[日本語](README.ja.md)

**Localize your clap CLI and rmcp MCP server with one set of `.ftl` files.**

lingokit is a thin wrapper around [Fluent](https://projectfluent.org/). Help texts, clap errors,
MCP tool descriptions and your own error messages all come from the same translation files.

| Crate | Role |
|---|---|
| [`lingokit`](https://docs.rs/lingokit) | Runtime: `Localizer`, locale resolution, built-in messages. Features `clap`, `rmcp`, `derive` (default) |
| [`lingokit-build`](https://docs.rs/lingokit-build) | `build.rs` helper: checks the FTL files and generates typed accessors |
| [`lingokit-macros`](https://docs.rs/lingokit-macros) | `#[derive(LocalizedError)]` (re-exported by `lingokit`) |

## Start from the template

Create a workspace with a library, a CLI and an MCP server with
[cargo-generate](https://github.com/cargo-generate/cargo-generate) (`template/`):

```sh
cargo generate gh:lovelygenerator/lingokit template --name my-tool
# without the CLI or the MCP server: -d with_cli=false / -d with_mcp=false
```

## Installation

```toml
[dependencies]
lingokit = { version = "0.1", features = ["clap", "rmcp"] }

[build-dependencies]
lingokit-build = "0.1"
```

```rust
// build.rs
fn main() {
    lingokit_build::Config::new("locales").generate(); // also .env_prefix(..) / .default_lang(..)
}
```

```rust
// lib.rs (crate root): defines LOCALES and the msg module
lingokit::include_locales!();
```

Put the FTL files in `locales/<locale>/*.ftl`. The default locale is `en`.

```ftl
# locales/en/main.ftl
err-not-found = File not found: { $path }
result-word-count = { $count ->
    [one] { $path } contains { $count } word.
   *[other] { $path } contains { $count } words.
}
```

## Errors

```rust
#[derive(Debug, thiserror::Error, lingokit::LocalizedError)]
pub enum Error {
    #[error("not found: {}", path.display())]   // English, for logs
    #[ftl("err-not-found")]                      // for display; the field becomes $path
    NotFound { path: PathBuf },
    #[error("io")]
    #[ftl("err-io")]
    Io(#[from] std::io::Error),                  // #[source] / #[from] fields are not passed
}

let text = loc.error(&err);                      // "ファイルが見つかりません: a.txt" in ja
```

- It is a **compile error** when the key is not in the FTL files, or when the message uses a
  variable that is not a field.
- Numbers stay numbers (so plurals work), paths use `display()`, everything else `Display`.
- Mark fields that should not be passed with `#[ftl(skip)]`.

For other messages, use the generated functions: `loc.msg(&msg::result_word_count(n, path))`.

## CLI (feature `clap`)

```rust
fn main() -> ExitCode {
    let (cli, loc) = lingokit::clap::parse::<Cli>(&LOCALES);
    lingokit::clap::exit_code(&loc, run(cli, &loc))   // errors are printed as "error: ..."
}
```

- A global `--lang` argument is added when your command does not define one.
- Help headings, the help/version arguments, the labels of defaults and possible values, and
  the common clap errors have built-in English and Japanese translations, which your FTL files
  can override.

| Target | Key |
|---|---|
| Command about | `cli-about` / `cli-<path>-about` (also `-long-about`) |
| Argument help | `cli-<path>-arg-<id>` (also `-long`), falling back to `cli-arg-<id>` |
| Possible value help | `cli-<path>-arg-<id>-value-<value>` |
| clap errors | `cli-error-<ErrorKind in kebab-case>` |

`<path>` is the chain of subcommand names joined with `-` (`cli-db-migrate-about`).

## MCP server (feature `rmcp`)

```rust
let tool_router = lingokit::rmcp::localize_if_enabled(Self::tool_router(), &loc);
let instructions = lingokit::rmcp::instructions(&loc, "English instructions");
```

- Tool descriptions and schemas stay as written in the source unless
  `<PREFIX>_MCP_LOCALIZE_SCHEMA=true` is set.
- Tool results and errors can always be localized with `loc.msg(..)` / `loc.error(..)`.

| Target | Key |
|---|---|
| Tool description | `mcp-tool-<name>` (`.title` attribute for the title) |
| Parameter | `mcp-tool-<name>-param-<prop>` |
| Instructions | `mcp-instructions` |

## Locale

`--lang` (CLI only) → `<PREFIX>_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG` → default locale

`<PREFIX>` defaults to the package name in upper case (`my-tool-core` → `MY_TOOL_CORE`) and can
be set with `Config::env_prefix`.

## Tests

```rust
#[test]
fn i18n_keys() {
    lingokit::clap::assert_keys::<Cli>(&LOCALES);                    // every CLI key
    lingokit::rmcp::assert_keys(&Server::tool_router(), &LOCALES);   // every MCP key
}
```

The test fails with the list of keys to add when the default locale lacks one.

## Build-time checks ([ftl-check](https://github.com/lovelygenerator/ftl-check))

`lingokit-build` runs ftl-check's `check` on every build.

| Diagnostics | Result |
|---|---|
| Severity `error` (syntax errors, missing translations E201, mismatched variables E211/E212, plural categories E302, ...) | The build fails |
| Severity `warn` (possibly untranslated W502, extra keys W202, ...) | Cargo warnings |

- An `ftl-check.toml`, searched for from `CARGO_MANIFEST_DIR` upward, configures the checks
  (`[rules]`, `[allow]`, ...).
- Its `path` and `base` are overridden by `Config::new(..)` / `default_lang(..)`, so the checked
  files are the ones the code is generated from.
- Running the `ftl-check` command with the same config gives the same result
  (`ftl-check explain E201` describes a code).
- Diagnostics are shown in English or Japanese: `FTL_CHECK_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG`.

## Notes

- The `rmcp` feature uses the public `ToolRouter::map` of rmcp, so rmcp is pinned to `~3.10`.
- Only the top-level `properties` of MCP input schemas are localized.
- The library API of ftl-check is unstable during 0.x, so it is pinned to `=0.1.0`.

## Minimum supported Rust version

1.88

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
