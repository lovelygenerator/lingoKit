# lingokit-macros

`#[derive(LocalizedError)]` for [lingokit](https://docs.rs/lingokit). Use it through
`lingokit::LocalizedError`.

```rust
#[derive(Debug, thiserror::Error, lingokit::LocalizedError)]
pub enum Error {
    #[error("not found: {}", path.display())]
    #[ftl("err-not-found")]
    NotFound { path: std::path::PathBuf },
}
```

Named fields become Fluent variables (`$path`). A missing key, or a variable that is not a
field, is a compile error.

License: MIT OR Apache-2.0
