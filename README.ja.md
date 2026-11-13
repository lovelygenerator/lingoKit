# lingokit

[English](README.md)

**CLI(clap)と MCP サーバー(rmcp)の多言語対応を、1組の `.ftl` ファイルで。**

lingokit は [Fluent](https://projectfluent.org/) の薄いラッパーです。help・clap のエラー・MCP ツールの説明・エラーメッセージを、同じ翻訳ファイルから言語ごとに出し分けます。

| クレート | 役割 |
|---|---|
| [`lingokit`](https://docs.rs/lingokit) | 実行時(`Localizer`・ロケール解決・組み込みメッセージ)。feature `clap` / `rmcp` / `derive`(既定で有効) |
| [`lingokit-build`](https://docs.rs/lingokit-build) | `build.rs` 用。FTL の検査と型付きアクセサの生成 |
| [`lingokit-macros`](https://docs.rs/lingokit-macros) | `#[derive(LocalizedError)]`(`lingokit` から再エクスポート) |

## テンプレートから始める

ライブラリ本体・CLI・MCP サーバーのワークスペースを [cargo-generate](https://github.com/cargo-generate/cargo-generate) で作れます(`template/`)。

```sh
cargo generate gh:lovelygenerator/lingokit template --name my-tool
# CLI や MCP が不要なら -d with_cli=false / -d with_mcp=false
```

## 導入

```toml
[dependencies]
lingokit = { version = "0.1", features = ["clap", "rmcp"] }

[build-dependencies]
lingokit-build = "0.1"
```

```rust
// build.rs
fn main() {
    lingokit_build::Config::new("locales").generate(); // .env_prefix(..) / .default_lang(..) も指定できる
}
```

```rust
// lib.rs(クレートのルート)。LOCALES と msg モジュールが定義される
lingokit::include_locales!();
```

FTL は `locales/<ロケール>/*.ftl` に置きます(既定言語は `en`)。

```ftl
# locales/ja/main.ftl
err-not-found = ファイルが見つかりません: { $path }
result-word-count = { $path } の単語数は { $count } です。
```

## エラー

```rust
#[derive(Debug, thiserror::Error, lingokit::LocalizedError)]
pub enum Error {
    #[error("not found: {}", path.display())]   // 英語(ログ用)
    #[ftl("err-not-found")]                      // 表示用。フィールドが $path に入る
    NotFound { path: PathBuf },
    #[error("io")]
    #[ftl("err-io")]
    Io(#[from] std::io::Error),                  // #[source] / #[from] は引数に渡さない
}

let text = loc.error(&err);                      // ja なら「ファイルが見つかりません: a.txt」
```

- キーが FTL にない、またはメッセージが使う変数がフィールドにない場合は**コンパイルエラー**
- 数値はそのまま(複数形の選択に使える)、パスは `display()`、それ以外は `Display` で渡す
- 渡したくないフィールドには `#[ftl(skip)]` を付ける

それ以外のメッセージは生成された関数で使います: `loc.msg(&msg::result_word_count(n, path))`

## CLI(feature `clap`)

```rust
fn main() -> ExitCode {
    let (cli, loc) = lingokit::clap::parse::<Cli>(&LOCALES);
    lingokit::clap::exit_code(&loc, run(cli, &loc))   // エラーは「エラー: ...」で表示
}
```

- `--lang` を定義していなければ、グローバル引数として自動で追加します
- help の見出し・help/version 引数・既定値と候補値のラベル・主な clap エラーは、英語と日本語の訳が組み込まれています(利用側の FTL で上書き可)

| 対象 | キー |
|---|---|
| コマンドの説明 | `cli-about` / `cli-<path>-about`(`-long-about` も可) |
| 引数の help | `cli-<path>-arg-<id>`(`-long` も可)、なければ `cli-arg-<id>` |
| 候補値の説明 | `cli-<path>-arg-<id>-value-<value>` |
| clap のエラー | `cli-error-<ErrorKind の kebab-case>` |

`<path>` はサブコマンドの連鎖を `-` でつないだものです(`cli-db-migrate-about`)。

## MCP サーバー(feature `rmcp`)

```rust
let tool_router = lingokit::rmcp::localize_if_enabled(Self::tool_router(), &loc);
let instructions = lingokit::rmcp::instructions(&loc, "English instructions");
```

- ツールの説明・スキーマは、`<PREFIX>_MCP_LOCALIZE_SCHEMA=true` のときだけ翻訳します(既定はソースに書いたまま)
- ツールの結果やエラーは `loc.msg(..)` / `loc.error(..)` でいつでも翻訳できます

| 対象 | キー |
|---|---|
| ツールの説明 | `mcp-tool-<name>`(`.title` 属性でタイトル) |
| パラメータ | `mcp-tool-<name>-param-<prop>` |
| instructions | `mcp-instructions` |

## 言語の決まり方

`--lang`(CLI のみ)→ `<PREFIX>_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG` → 既定言語

`<PREFIX>` は既定でパッケージ名の大文字(`my-tool-core` → `MY_TOOL_CORE`)で、`Config::env_prefix` で変更できます。

## テスト

```rust
#[test]
fn i18n_keys() {
    lingokit::clap::assert_keys::<Cli>(&LOCALES);                    // CLI の全キー
    lingokit::rmcp::assert_keys(&Server::tool_router(), &LOCALES);   // MCP の全キー
}
```

既定言語にキーがなければ、追加すべきキーの一覧を表示して失敗します。

## ビルド時チェック([ftl-check](https://github.com/lovelygenerator/ftl-check))

`lingokit-build` はビルドのたびに ftl-check の `check` を実行します。

| 診断 | 結果 |
|---|---|
| 重大度 error(構文エラー・翻訳の欠落 E201・引数の不一致 E211/E212・複数形 E302 など) | ビルド失敗 |
| 重大度 warn(未翻訳の疑い W502・余分なキー W202 など) | cargo の警告 |

- `ftl-check.toml` があれば(`CARGO_MANIFEST_DIR` から上位へ探す)`[rules]` / `[allow]` などに従います
- `path` と `base` は `Config::new(..)` / `default_lang(..)` の値で上書きするので、コード生成と同じファイルを検査します
- 同じ設定で `ftl-check` コマンドを実行すると、ビルドと同じ結果になります(`ftl-check explain E201` でコードの説明)
- 診断の表示言語は `FTL_CHECK_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG`(英語・日本語)

## 注意

- feature `rmcp` は rmcp の `ToolRouter::map`(pub)を使うため、rmcp を `~3.10` に固定しています
- MCP の入力スキーマは、トップレベルの `properties` だけを翻訳します
- ftl-check のライブラリ API は 0.x の間は不安定なため、`=0.1.0` に固定しています

## 最小サポート Rust バージョン

1.88

## ライセンス

[Apache License, Version 2.0](LICENSE-APACHE) または [MIT license](LICENSE-MIT) のいずれかを選択できます。
