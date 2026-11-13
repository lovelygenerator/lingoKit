//! lingokit の build.rs 用ヘルパー。
//!
//! `<dir>/<lang>/*.ftl` を検証し、`OUT_DIR/lingokit_locales.rs` を生成する。
//! 生成物はクレートのルートで `lingokit::include_locales!()` により展開する。
//!
//! ```ignore
//! // build.rs
//! fn main() {
//!     lingokit_build::Config::new("locales").generate();
//! }
//! ```
//!
//! 生成されるもの:
//! - `pub static LOCALES: lingokit::Locales`
//! - `pub mod msg`: メッセージごとの型付きアクセサ(`msg::err_not_found(path)` など)
//!
//! ビルド時チェックは [ftl-check](https://crates.io/crates/ftl-check) で行う:
//! - 重大度 error の診断(構文エラー、翻訳の欠落、引数の不一致など) → ビルド失敗
//! - 重大度 warn の診断 → `cargo:warning`
//!
//! `ftl-check.toml` があれば(`CARGO_MANIFEST_DIR` から上位へ探す)その `[rules]` などに従う。
//! `path` と `base` は lingokit-build の設定で上書きするので、コード生成と同じファイルを検査する。
//! 同じ設定ファイルで CLI の `ftl-check` を実行すれば、ビルドと同じ結果になる。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

use fluent_syntax::{ast, parser};

/// メッセージ ID → 参照している変数名
type Messages = BTreeMap<String, BTreeSet<String>>;

struct Lang {
    name: String,
    files: Vec<PathBuf>,
    messages: Messages,
}

/// 生成の設定。
#[derive(Debug, Clone)]
pub struct Config {
    dir: PathBuf,
    default_lang: String,
    env_prefix: Option<String>,
}

impl Config {
    /// `dir` は `CARGO_MANIFEST_DIR` からの相対パス(絶対パスも可)。
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), default_lang: "en".into(), env_prefix: None }
    }

    /// 最終フォールバック言語(既定: `en`)。`<dir>/<lang>/` が必須。
    pub fn default_lang(mut self, lang: impl Into<String>) -> Self {
        self.default_lang = lang.into();
        self
    }

    /// 環境変数の接頭辞(既定: パッケージ名の大文字。`-` は `_`)。
    /// 例: `my-tool-core` → `MY_TOOL_CORE`。`<PREFIX>_LANG` などに使う。
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// 検証して生成する。失敗したら panic(= ビルド失敗)。
    pub fn generate(self) {
        if let Err(e) = self.try_generate() {
            panic!("{e}");
        }
    }

    pub fn try_generate(self) -> Result<(), String> {
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?);
        let dir = manifest.join(&self.dir);
        let dir = dir.canonicalize().map_err(|e| format!("{} が見つかりません: {e}", dir.display()))?;
        println!("cargo:rerun-if-changed={}", dir.display());
        for k in ["FTL_CHECK_LANG", "LC_ALL", "LC_MESSAGES", "LANG"] {
            println!("cargo:rerun-if-env-changed={k}"); // ftl-check の表示言語
        }

        let env_prefix = self.env_prefix.clone().unwrap_or_else(|| {
            env::var("CARGO_PKG_NAME").unwrap_or_default().to_ascii_uppercase().replace('-', "_")
        });

        check(&manifest, &dir, &self.default_lang)?;

        let langs = load(&dir)?;
        let default = langs
            .iter()
            .find(|l| l.name == self.default_lang)
            .ok_or_else(|| format!("既定言語のフォルダ {}/{}/ がありません", dir.display(), self.default_lang))?;

        let out = PathBuf::from(env::var("OUT_DIR").map_err(|e| e.to_string())?).join("lingokit_locales.rs");
        fs::write(out, render(&langs, default, &self.default_lang, &env_prefix)).map_err(|e| e.to_string())
    }
}

/// ftl-check で検査する。error があれば Err、warn は cargo:warning に出す。
fn check(manifest: &Path, dir: &Path, base: &str) -> Result<(), String> {
    use ftl_check::diag::Severity;
    use ftl_check::output::{self, TextStyle};
    use ftl_check::run::{self, Command, Options};

    // 設定ファイルの場所(path はその場所からの相対で指定する)
    let loaded = ftl_check::project::config::load(None, manifest).map_err(|e| format!("ftl-check.toml: {e:?}"))?;
    if let Some(f) = &loaded.file {
        println!("cargo:rerun-if-changed={}", f.display());
    }
    let root = loaded.root.canonicalize().unwrap_or(loaded.root.clone());
    let rel = dir.strip_prefix(&root).map(Path::to_path_buf).unwrap_or_else(|_| dir.to_path_buf());
    let pattern = format!("{}/{{locale}}/**/*.ftl", rel.display().to_string().replace('\\', "/"));

    let opts = Options {
        cwd: manifest.to_path_buf(),
        path: Some(pattern),
        base: Some(base.to_string()),
        ..Options::default()
    };
    let report = run::run(Command::Check, &opts).map_err(|f| format!("ftl-check を実行できませんでした: {:?}", f.problem))?;

    let lang = ftl_check::i18n::choose_lang(&ftl_check::i18n::LangInputs {
        cli: None,
        env: &|k| env::var(k).ok(),
        system: &|| None,
        json: false,
    });
    let loc = ftl_check::i18n::Localizer::new(&lang.lang);
    let text = output::text(&report, &loc, TextStyle { color: false, quiet: false });

    let has_warn = report.diagnostics.iter().any(|d| d.severity == Severity::Warn);
    if report.ok {
        if has_warn {
            for line in text.lines().filter(|l| !l.trim().is_empty()) {
                println!("cargo:warning=[ftl-check] {line}");
            }
        }
        Ok(())
    } else {
        Err(format!("ftl-check で FTL のエラーが見つかりました(詳細は `ftl-check explain <コード>`):\n\n{text}"))
    }
}

/// コード生成用に読み込む(検査は ftl-check で済んでいる)
fn load(dir: &Path) -> Result<Vec<Lang>, String> {
    let mut langs = Vec::new();
    for ldir in sorted_entries(dir)?.into_iter().filter(|p| p.is_dir()) {
        println!("cargo:rerun-if-changed={}", ldir.display());
        let name = ldir.file_name().unwrap().to_string_lossy().into_owned();
        let mut lang = Lang { name, files: Vec::new(), messages: Messages::new() };
        for file in sorted_entries(&ldir)?.into_iter().filter(|p| p.extension().is_some_and(|e| e == "ftl")) {
            println!("cargo:rerun-if-changed={}", file.display());
            let src = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let res = parser::parse(src.as_str()).unwrap_or_else(|(res, _)| res);
            collect_messages(&res, &mut lang.messages);
            lang.files.push(file);
        }
        langs.push(lang);
    }
    Ok(langs)
}

fn sorted_entries(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut v: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    v.sort();
    Ok(v)
}

fn collect_messages(res: &ast::Resource<&str>, out: &mut Messages) {
    for entry in &res.body {
        let ast::Entry::Message(m) = entry else { continue };
        let id = m.id.name.to_string();
        for a in &m.attributes {
            let mut avars = BTreeSet::new();
            pattern_vars(&a.value, &mut avars);
            out.insert(format!("{id}.{}", a.id.name), avars);
        }
        if let Some(p) = &m.value {
            let mut vars = BTreeSet::new();
            pattern_vars(p, &mut vars);
            out.insert(id, vars);
        }
    }
}

fn pattern_vars(p: &ast::Pattern<&str>, vars: &mut BTreeSet<String>) {
    for el in &p.elements {
        if let ast::PatternElement::Placeable { expression } = el {
            expr_vars(expression, vars);
        }
    }
}

fn expr_vars(e: &ast::Expression<&str>, vars: &mut BTreeSet<String>) {
    match e {
        ast::Expression::Inline(i) => inline_vars(i, vars),
        ast::Expression::Select { selector, variants } => {
            inline_vars(selector, vars);
            for v in variants {
                pattern_vars(&v.value, vars);
            }
        }
    }
}

fn inline_vars(i: &ast::InlineExpression<&str>, vars: &mut BTreeSet<String>) {
    match i {
        ast::InlineExpression::VariableReference { id } => {
            vars.insert(id.name.to_string());
        }
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline_vars(a, vars);
            }
            for a in &arguments.named {
                inline_vars(&a.value, vars);
            }
        }
        ast::InlineExpression::Placeable { expression } => expr_vars(expression, vars),
        _ => {}
    }
}

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "static", "struct", "trait", "true", "type",
    "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

/// キー/変数名 → Rust の識別子(lingokit-macros と同じ規則)
fn rust_ident(s: &str) -> String {
    let s = s.replace(['-', '.'], "_");
    if KEYWORDS.contains(&s.as_str()) { format!("r#{s}") } else { s }
}

fn render(langs: &[Lang], default: &Lang, default_lang: &str, env_prefix: &str) -> String {
    let mut o = String::from("// @generated by lingokit-build — 編集しないでください\n\n");
    o.push_str("/// 翻訳リソース一式(lingokit-build が生成)\n");
    o.push_str("pub static LOCALES: ::lingokit::Locales = ::lingokit::Locales {\n");
    let _ = writeln!(o, "    default_lang: {default_lang:?},\n    env_prefix: {env_prefix:?},\n    resources: &[");
    for l in langs {
        let _ = writeln!(o, "        ({:?}, &[", l.name);
        for f in &l.files {
            let fname = f.file_name().unwrap().to_string_lossy();
            let _ = writeln!(o, "            ({fname:?}, include_str!({:?})),", f.display().to_string());
        }
        o.push_str("        ]),\n");
    }
    o.push_str("    ],\n    ids: &[\n");
    for id in default.messages.keys() {
        let _ = writeln!(o, "        {id:?},");
    }
    o.push_str("    ],\n};\n\n");

    o.push_str("/// メッセージごとの型付きアクセサ(lingokit-build が生成)\n");
    o.push_str("#[allow(dead_code, clippy::all)]\npub mod msg {\n    use ::lingokit::{FluentArgs, FluentValue, Msg};\n\n");
    for (id, vars) in &default.messages {
        let fname = rust_ident(id);
        let _ = writeln!(o, "    /// `{id}`");
        if vars.is_empty() {
            let _ = writeln!(o, "    pub fn {fname}() -> Msg<'static> {{ Msg::new({id:?}) }}\n");
            continue;
        }
        let params: Vec<String> =
            vars.iter().map(|v| format!("{}: impl Into<FluentValue<'a>>", rust_ident(v))).collect();
        let _ = writeln!(o, "    pub fn {fname}<'a>({}) -> Msg<'a> {{", params.join(", "));
        o.push_str("        let mut args = FluentArgs::new();\n");
        for v in vars {
            let _ = writeln!(o, "        args.set({v:?}, {}.into());", rust_ident(v));
        }
        let _ = writeln!(o, "        Msg::with_args({id:?}, args)\n    }}\n");
    }
    o.push_str("    /// derive(LocalizedError) の検査用: メッセージが使う変数\n");
    o.push_str("    #[doc(hidden)]\n    #[allow(non_upper_case_globals)]\n    pub mod __vars {\n");
    for (id, vars) in &default.messages {
        let list: Vec<String> = vars.iter().map(|v| format!("{v:?}")).collect();
        let _ = writeln!(o, "        pub const {}: &[&str] = &[{}];", rust_ident(id), list.join(", "));
    }
    o.push_str("    }\n}\n");
    o
}
