# lingokit-build

`build.rs` helper for [lingokit](https://docs.rs/lingokit).

```rust
// build.rs
fn main() {
    lingokit_build::Config::new("locales").generate();
}
```

- Checks the FTL files in `locales/<locale>/**/*.ftl` with
  [ftl-check](https://crates.io/crates/ftl-check): errors fail the build, warnings are printed.
- Generates `LOCALES` and a typed function for each message (`msg::err_not_found(path)`),
  included with `lingokit::include_locales!()`.

See the [repository](https://github.com/lovelygenerator/lingokit) for the full guide.

License: MIT OR Apache-2.0
