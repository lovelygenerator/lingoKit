//! clap の CLI を FTL のキー規約に従って翻訳する(feature `clap`)。
//!
//! | 対象 | キー |
//! |---|---|
//! | コマンドの about | `cli-about` / `cli-<path>-about`(`-long-about` も可) |
//! | 引数の help | `cli-<path>-arg-<id>`(`-long` も可)。なければ `cli-arg-<id>` |
//! | 候補値の説明 | `cli-<path>-arg-<id>-value-<value>` |
//! | 見出し | `cli-heading-{usage,arguments,options,commands}` |
//! | ラベル | `cli-label-{possible-values,default}` |
//! | clap のエラー | `cli-error-<kind>`(引数: `$arg` `$value` `$valid_values` `$suggested` `$subcommand`) |
//!
//! `<path>` はサブコマンドの連鎖を `-` でつないだもの。キーがなければ元の(英語の)文言のまま。
//! 見出し・ラベル・help/version/lang 引数・主なエラーは lingokit の組み込み(en/ja)があり、上書きもできる。

use std::ffi::OsString;
use std::process::ExitCode;

use clap::error::{ContextKind, ErrorKind};
use clap::{Arg, Command, CommandFactory, FromArgMatches};

use crate::{FluentArgs, Locales, LocalizedError, Localizer, render_error_chain};

/// argv を解析し、翻訳済みの `Localizer` と一緒に返す。
///
/// - `--lang` を事前に拾って言語を決める(`--lang` が未定義なら自動で追加する)
/// - help・エラーを翻訳して表示し、必要なら終了する
pub fn parse<C: CommandFactory + FromArgMatches>(locales: &'static Locales) -> (C, Localizer) {
    parse_from(locales, std::env::args_os())
}

/// [`parse`] の引数指定版(テスト用)。
pub fn parse_from<C, I, T>(locales: &'static Locales, args: I) -> (C, Localizer)
where
    C: CommandFactory + FromArgMatches,
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let loc = Localizer::from_env(locales, crate::locale::pre_scan_lang(args.clone()).as_deref());
    let cmd = command::<C>(&loc);
    let m = cmd.try_get_matches_from(args).unwrap_or_else(|e| exit_with_clap_error(e, &loc));
    let c = C::from_arg_matches(&m).unwrap_or_else(|e| exit_with_clap_error(e, &loc));
    (c, loc)
}

/// 翻訳済みの `Command` を作る(`--lang` が未定義なら追加する)。
pub fn command<C: CommandFactory>(loc: &Localizer) -> Command {
    localize_command(with_lang_arg(C::command()), loc)
}

fn with_lang_arg(cmd: Command) -> Command {
    if cmd.get_arguments().any(|a| a.get_id() == "lang") {
        return cmd;
    }
    cmd.arg(
        Arg::new("lang")
            .long("lang")
            .global(true)
            .value_name("LANG")
            .help("Display language (e.g. en, ja)"),
    )
}

/// エラーを「エラー: メッセージ: 原因」の形で stderr に出し、失敗の終了コードを返す。
pub fn report(loc: &Localizer, e: &dyn LocalizedError) -> ExitCode {
    eprintln!("{}: {}", loc.fmt("cli-error-label", None), render_error_chain(loc, e));
    ExitCode::FAILURE
}

/// `Result` をそのまま終了コードにする。エラーなら [`report`] する。
pub fn exit_code<E: LocalizedError>(loc: &Localizer, r: Result<(), E>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => report(loc, &e),
    }
}

/// CLI の全コマンド・引数・候補値に対応するキーが既定言語にあるかを検査する(テスト用)。
pub fn assert_keys<C: CommandFactory>(locales: &'static Locales) {
    crate::testing::assert_required_keys(locales, &required_keys(&with_lang_arg(C::command())));
}

struct Ctx<'a> {
    loc: &'a Localizer,
    template: Option<String>,
    h_options: Option<String>,
    h_arguments: Option<String>,
    h_commands: Option<String>,
    l_possible: String,
    l_default: String,
}

/// コマンドツリー全体を翻訳する。ビルド済み(help/version 引数や help サブコマンドが生成済み)の状態で書き換える。
pub fn localize_command(mut cmd: Command, loc: &Localizer) -> Command {
    cmd.build();
    let styles = cmd.get_styles();
    let template = loc.get("cli-heading-usage", None).map(|usage| {
        let s = styles.get_usage();
        format!(
            "{{before-help}}{{about-with-newline}}\n{}{usage}:{} {{usage}}\n\n{{all-args}}{{after-help}}",
            s.render(),
            s.render_reset()
        )
    });
    let ctx = Ctx {
        loc,
        template,
        h_options: loc.get("cli-heading-options", None),
        h_arguments: loc.get("cli-heading-arguments", None),
        h_commands: loc.get("cli-heading-commands", None),
        l_possible: loc.get("cli-label-possible-values", None).unwrap_or_else(|| "possible values".into()),
        l_default: loc.get("cli-label-default", None).unwrap_or_else(|| "default".into()),
    };
    walk(cmd, "", &ctx)
}

fn prefix(path: &str) -> String {
    if path.is_empty() { "cli".into() } else { format!("cli-{path}") }
}

fn walk(mut cmd: Command, path: &str, ctx: &Ctx) -> Command {
    let p = prefix(path);
    if let Some(s) = ctx.loc.get(&format!("{p}-about"), None) {
        cmd = cmd.about(s);
    }
    if let Some(s) = ctx.loc.get(&format!("{p}-long-about"), None) {
        cmd = cmd.long_about(s);
    }
    if let Some(t) = &ctx.template {
        cmd = cmd.help_template(t.clone());
    }
    if let Some(h) = &ctx.h_commands {
        cmd = cmd.subcommand_help_heading(h.clone());
    }

    let ids: Vec<String> = cmd.get_arguments().map(|a| a.get_id().to_string()).collect();
    for id in ids {
        cmd = cmd.mut_arg(&id, |a| localize_arg(a, &p, ctx));
    }

    let subs: Vec<String> = cmd.get_subcommands().map(|s| s.get_name().to_string()).collect();
    for name in subs {
        // 自動生成の `help` サブコマンドの子(`help count` 用)は、本体のサブコマンドと同じキーを使う
        let sub_path = if path.is_empty() || path == "help" { name.clone() } else { format!("{path}-{name}") };
        cmd = cmd.mut_subcommand(&name, |s| walk(s, &sub_path, ctx));
    }
    cmd
}

/// 引数のキー: `<prefix>-arg-<id><suffix>`、なければ `cli-arg-<id><suffix>`(グローバル引数・help/version 用)
fn arg_text(loc: &Localizer, p: &str, id: &str, suffix: &str) -> Option<String> {
    loc.get(&format!("{p}-arg-{id}{suffix}"), None).or_else(|| loc.get(&format!("cli-arg-{id}{suffix}"), None))
}

fn localize_arg(mut a: Arg, p: &str, ctx: &Ctx) -> Arg {
    let loc = ctx.loc;
    let id = a.get_id().to_string();

    let help = arg_text(loc, p, &id, "").or_else(|| a.get_help().map(|s| s.to_string()));
    // 短い help だけ翻訳されている場合は、英語の long help を使わず短い help を使う(言語が混ざらないように)
    let long = arg_text(loc, p, &id, "-long").or_else(|| match arg_text(loc, p, &id, "") {
        Some(_) => None,
        None => a.get_long_help().map(|s| s.to_string()),
    });
    let mut short_extra = String::new();
    let mut long_extra = String::new();

    // 候補値(value_enum など): clap の表示を隠し、翻訳した一覧を help に足す
    let pvs: Vec<_> = a.get_possible_values().into_iter().filter(|v| !v.is_hide_set()).collect();
    if !pvs.is_empty() && !a.is_hide_possible_values_set() {
        let names: Vec<&str> = pvs.iter().map(|v| v.get_name()).collect();
        short_extra.push_str(&format!(" [{}: {}]", ctx.l_possible, names.join(", ")));
        long_extra.push_str(&format!("\n\n{}:", ctx.l_possible));
        for v in &pvs {
            let name = v.get_name();
            let vhelp = arg_text(loc, p, &id, &format!("-value-{name}")).or_else(|| v.get_help().map(|s| s.to_string()));
            match vhelp {
                Some(h) => long_extra.push_str(&format!("\n  - {name}: {h}")),
                None => long_extra.push_str(&format!("\n  - {name}")),
            }
        }
        a = a.hide_possible_values(true);
    }

    // 既定値の「[default: x]」もラベルを翻訳する
    let defaults: Vec<String> = a.get_default_values().iter().map(|v| v.to_string_lossy().into_owned()).collect();
    if !defaults.is_empty() && !a.is_hide_default_value_set() && a.get_action().takes_values() {
        let d = format!(" [{}: {}]", ctx.l_default, defaults.join(", "));
        short_extra.push_str(&d);
        long_extra.insert_str(0, &d);
        a = a.hide_default_value(true);
    }

    if let Some(h) = &help {
        a = a.help(format!("{h}{short_extra}"));
    }
    if long.is_some() || !long_extra.is_empty() {
        let base = long.or(help).unwrap_or_default();
        a = a.long_help(format!("{base}{long_extra}"));
    }

    if a.get_help_heading().is_none() {
        let h = if a.is_positional() { &ctx.h_arguments } else { &ctx.h_options };
        if let Some(h) = h {
            a = a.help_heading(h.clone());
        }
    }
    a
}

/// clap のエラーを翻訳して表示し、終了する。対応するキーがなければ clap の英語の表示のまま。
pub fn exit_with_clap_error(e: clap::Error, loc: &Localizer) -> ! {
    use ErrorKind::*;
    if matches!(e.kind(), DisplayHelp | DisplayVersion | DisplayHelpOnMissingArgumentOrSubcommand) {
        e.exit();
    }
    let key = format!("cli-error-{}", kebab(&format!("{:?}", e.kind())));
    let mut args = FluentArgs::new();
    for (k, v) in e.context() {
        let name = match k {
            ContextKind::InvalidArg => "arg",
            ContextKind::InvalidValue => "value",
            ContextKind::ValidValue => "valid_values",
            ContextKind::SuggestedArg | ContextKind::SuggestedValue | ContextKind::SuggestedSubcommand => "suggested",
            ContextKind::InvalidSubcommand => "subcommand",
            _ => continue,
        };
        args.set(name, v.to_string());
    }
    let Some(text) = loc.get(&key, Some(&args)) else { e.exit() };

    let label = loc.fmt("cli-error-label", None);
    let mut out = format!("{label}: {text}\n");
    if let Some(usage) = e.get(ContextKind::Usage) {
        let usage = usage.to_string();
        let usage = match loc.get("cli-heading-usage", None) {
            Some(h) => usage.replacen("Usage:", &format!("{h}:"), 1),
            None => usage,
        };
        out.push_str(&format!("\n{usage}\n"));
    }
    if let Some(hint) = loc.get("cli-error-hint", None) {
        out.push_str(&format!("\n{hint}\n"));
    }
    eprint!("{out}");
    std::process::exit(e.exit_code());
}

fn kebab(s: &str) -> String {
    let mut o = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                o.push('-');
            }
            o.push(c.to_ascii_lowercase());
        } else {
            o.push(c);
        }
    }
    o
}

/// 必須キーの一覧(テスト用)。各要素は「どれか1つがあればよい」候補。
pub fn required_keys(cmd: &Command) -> Vec<Vec<String>> {
    let mut cmd = cmd.clone();
    cmd.build();
    let mut out = Vec::new();
    collect(&cmd, "", &mut out);
    out
}

fn collect(cmd: &Command, path: &str, out: &mut Vec<Vec<String>>) {
    let p = prefix(path);
    out.push(vec![format!("{p}-about")]);
    for a in cmd.get_arguments() {
        let id = a.get_id().to_string();
        out.push(vec![format!("{p}-arg-{id}"), format!("cli-arg-{id}")]);
        for v in a.get_possible_values().iter().filter(|v| !v.is_hide_set()) {
            let n = v.get_name();
            out.push(vec![format!("{p}-arg-{id}-value-{n}"), format!("cli-arg-{id}-value-{n}")]);
        }
    }
    if path == "help" {
        return; // 子は本体のサブコマンドのキーを流用する
    }
    for s in cmd.get_subcommands() {
        let sp = if path.is_empty() { s.get_name().to_string() } else { format!("{path}-{}", s.get_name()) };
        collect(s, &sp, out);
    }
}
