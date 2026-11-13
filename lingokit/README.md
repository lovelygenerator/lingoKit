# lingokit

Localize [clap](https://docs.rs/clap) CLIs and [rmcp](https://docs.rs/rmcp) MCP servers with
[Fluent](https://projectfluent.org/): one set of `.ftl` files for help texts, clap errors, MCP
tool descriptions and your own messages.

```rust
// build.rs: lingokit_build::Config::new("locales").generate();
lingokit::include_locales!();

#[derive(Debug, thiserror::Error, lingokit::LocalizedError)]
pub enum Error {
    #[error("not found: {}", path.display())]
    #[ftl("err-not-found")]
    NotFound { path: std::path::PathBuf },
}

let (cli, loc) = lingokit::clap::parse::<Cli>(&LOCALES);
```

Features: `clap`, `rmcp`, `derive` (default).

See the [repository](https://github.com/lovelygenerator/lingokit) for the full guide and a
cargo-generate template.

License: MIT OR Apache-2.0
