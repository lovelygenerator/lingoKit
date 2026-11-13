# lingokit

**CLI(clap)と MCP サーバー(rmcp)の多言語対応を、1組の `.ftl` ファイルで。**

[Fluent](https://projectfluent.org/) の薄いラッパーです。help・clap のエラー・MCP ツールの説明・エラーメッセージを、同じ翻訳ファイルから言語ごとに出し分けます。

| クレート | 役割 |
|---|---|
| `lingokit` | 実行時(Localizer・ロケール解決)。feature `clap` / `rmcp` / `derive`(既定で有効) |
| `lingokit-build` | build.rs 用。FTL の検証と型付きアクセサの生成 |
| `lingokit-macros` | `#[derive(LocalizedError)]` |

## 導入

```toml
[dependencies]
lingokit = { git = "https://github.com/lovelygenerator/lingokit", features = ["clap", "rmcp"] }

[build-dependencies]
lingokit-build = { git = "https://github.com/lovelygenerator/lingokit" }
```

```rust
// build.rs
fn main() {
    lingokit_build::Config::new("locales").generate(); // env_prefix(...) / default_lang(...) も指定できる
}
```

```rust
// lib.rs(クレートのルート)。LOCALES と msg モジュールが定義される
lingokit::include_locales!();
```

FTL は `locales/<lang>/*.ftl` に置きます(既定言語は `en`)。

## エラー

```rust
#[derive(Debug, thiserror::Error, lingokit::LocalizedError)]
pub enum Error {
    #[error("not found: {}", path.display())]   // 英語(ログ用)
    #[ftl("err-not-found")]                      // 表示用。フィールドが $path に入る
    NotFound { path: PathBuf },
    #[error("io")]
    #[ftl("err-io")]
    Io(#[from] std::io::Error),                  // #[source]/#[from] は引数に渡さない
}
```

- キーが FTL にない、またはメッセージが使う変数がフィールドにない → **コンパイルエラー**
- 数値はそのまま(複数形の選択に使える)、パスは `display()`、それ以外は `Display` で渡す
- 渡したくないフィールドには `#[ftl(skip)]` を付ける

キーを直接使うときは生成アクセサを使います: `loc.msg(&msg::err_not_found(path))`

## CLI(feature `clap`)

```rust
fn main() -> ExitCode {
    let (cli, loc) = lingokit::clap::parse::<Cli>(&LOCALES);
    lingokit::clap::exit_code(&loc, run(cli, &loc))   // エラーは「エラー: ...」で表示
}
```

- `--lang` は未定義なら自動で追加します(グローバル引数)
- help の見出し・help/version 引数・既定値と候補値のラベル・主な clap エラーは組み込みの en/ja 訳があります(利用側の FTL で上書き可)

| 対象 | キー |
|---|---|
| コマンドの説明 | `cli-about` / `cli-<path>-about`(`-long-about` も可) |
| 引数の help | `cli-<path>-arg-<id>`(`-long` も可)、なければ `cli-arg-<id>` |
| 候補値の説明 | `cli-<path>-arg-<id>-value-<value>` |
| clap のエラー | `cli-error-<ErrorKind の kebab-case>` |

## MCP(feature `rmcp`)

```rust
let tool_router = lingokit::rmcp::localize_if_enabled(Self::tool_router(), &loc);
let instructions = lingokit::rmcp::instructions(&loc, "English instructions");
```

- ツールの説明・スキーマは既定では英語のまま。`<PREFIX>_MCP_LOCALIZE_SCHEMA=true` で翻訳する
- 結果やエラーの文言は `loc.msg(...)` / `loc.error(...)` で常に翻訳できる

| 対象 | キー |
|---|---|
| ツールの説明 | `mcp-tool-<name>`(`.title` 属性でタイトル) |
| パラメータ | `mcp-tool-<name>-param-<prop>` |
| instructions | `mcp-instructions` |

## 言語の決まり方

`--lang`(CLI のみ)→ `<PREFIX>_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG` → 既定言語

`<PREFIX>` は既定でパッケージ名の大文字(`my-tool-core` → `MY_TOOL_CORE`)。`Config::env_prefix` で変更できます。

## テスト

```rust
#[test]
fn i18n_keys() {
    lingokit::clap::assert_keys::<Cli>(&LOCALES);                    // CLI の全キー
    lingokit::rmcp::assert_keys(&Server::tool_router(), &LOCALES);   // MCP の全キー
}
```

既定言語にキーがなければ失敗し、追加すべきキーの一覧を表示します。他の言語の欠落は警告のみです。

## ビルド時チェック([ftl-check](https://github.com/lovelygenerator/ftl-check))

`lingokit-build` はビルドのたびに ftl-check の `check` を実行します。

| 診断 | 結果 |
|---|---|
| 重大度 error(構文エラー・翻訳の欠落 E201・引数の不一致 E211/E212・複数形 E302 など) | ビルド失敗 |
| 重大度 warn(未翻訳の疑い W502・余分なキー W202 など) | `cargo:warning` |

- `ftl-check.toml` があれば(`CARGO_MANIFEST_DIR` から上位へ探す)`[rules]` / `[allow]` などに従います
- `path` と `base` は `Config::new(..)` / `default_lang(..)` の値で上書きするので、コード生成と同じファイルを検査します
- 同じ設定で CLI の `ftl-check` を実行すると、ビルドと同じ結果になります(`ftl-check explain E201` でコードの説明)
- 表示言語は `FTL_CHECK_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG`(英語・日本語)

## 注意

- rmcp は `ToolRouter.map`(pub)に依存しているため `~3.10` に固定しています
- MCP のスキーマ翻訳はトップレベルの `properties` だけが対象です
- `fluent-langneg` は fluent-bundle 0.16 に合わせて 0.13 を使っています
- ftl-check のライブラリ API は 0.x の間不安定なため `=0.1.0` で固定しています。MSRV は ftl-check に合わせて 1.88 です

## ライセンス

[MIT](LICENSE-MIT) または [Apache-2.0](LICENSE-APACHE) のいずれかを選択できます。
