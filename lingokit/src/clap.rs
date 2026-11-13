//! Localizing [clap](https://docs.rs/clap) CLIs (feature `clap`).
//!
//! Texts are looked up by these keys:
//!
//! | Target | Key |
//! |---|---|
//! | Command about | `cli-about` / `cli-<path>-about` (also `-long-about`) |
//! | Argument help | `cli-<path>-arg-<id>` (also `-long`), falling back to `cli-arg-<id>` |
//! | Possible value help | `cli-<path>-arg-<id>-value-<value>` |
//! | Help headings | `cli-heading-{usage,arguments,options,commands}` |
//! | Labels | `cli-label-{possible-values,default}` |
//! | clap errors | `cli-error-<kind>` (variables: `$arg` `$value` `$valid_values` `$suggested` `$subcommand`) |
//!
//! `<path>` is the chain of subcommand names joined with `-`. When a key is missing, the text
//! written in the source code is kept.
//! Headings, labels, the help/version/lang arguments and the common errors have built-in
//! translations (en, ja), which your FTL files can override.

use std::ffi::OsString;
use std::process::ExitCode;

use clap::error::{ContextKind, ErrorKind};
use clap::{Arg, Command, CommandFactory, FromArgMatches};

use crate::{FluentArgs, Locales, LocalizedError, Localizer, render_error_chain};

/// Parses `std::env::args_os()` and returns the arguments together with the [`Localizer`].
///
/// - The display language is taken from `--lang` before parsing. A global `--lang` argument is
///   added when the command does not define one.
/// - Help and errors are printed localized, and the process exits when clap would exit.
pub fn parse<C: CommandFactory + FromArgMatches>(locales: &'static Locales) -> (C, Localizer) {
    parse_from(locales, std::env::args_os())
}

/// [`parse`] with explicit arguments.
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

/// Builds the localized [`Command`] of `C`, adding `--lang` when it is not defined.
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

/// Prints `<error label>: <message>: <causes>` to stderr and returns [`ExitCode::FAILURE`].
pub fn report(loc: &Localizer, e: &dyn LocalizedError) -> ExitCode {
    eprintln!("{}: {}", loc.fmt("cli-error-label", None), render_error_chain(loc, e));
    ExitCode::FAILURE
}

/// Turns a `Result` into an exit code, [`report`]ing the error if there is one.
pub fn exit_code<E: LocalizedError>(loc: &Localizer, r: Result<(), E>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => report(loc, &e),
    }
}

/// Asserts that the default locale has a key for every command, argument and possible value
/// of `C`. Intended for tests.
///
/// # Panics
///
/// When keys are missing, with the list of keys to add.
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

/// Localizes a whole command tree.
///
/// The command is built first, so that the generated help/version arguments and the `help`
/// subcommand are localized as well.
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
        // The children of the generated `help` subcommand (`help <sub>`) use the keys of the real subcommands
        let sub_path = if path.is_empty() || path == "help" { name.clone() } else { format!("{path}-{name}") };
        cmd = cmd.mut_subcommand(&name, |s| walk(s, &sub_path, ctx));
    }
    cmd
}

/// `<prefix>-arg-<id><suffix>`, falling back to `cli-arg-<id><suffix>` (global arguments, help/version)
fn arg_text(loc: &Localizer, p: &str, id: &str, suffix: &str) -> Option<String> {
    loc.get(&format!("{p}-arg-{id}{suffix}"), None).or_else(|| loc.get(&format!("cli-arg-{id}{suffix}"), None))
}

fn localize_arg(mut a: Arg, p: &str, ctx: &Ctx) -> Arg {
    let loc = ctx.loc;
    let id = a.get_id().to_string();

    let help = arg_text(loc, p, &id, "").or_else(|| a.get_help().map(|s| s.to_string()));
    // When only the short help is translated, use it as the long help too, so that languages do not mix
    let long = arg_text(loc, p, &id, "-long").or_else(|| match arg_text(loc, p, &id, "") {
        Some(_) => None,
        None => a.get_long_help().map(|s| s.to_string()),
    });
    let mut short_extra = String::new();
    let mut long_extra = String::new();

    // Possible values (value_enum etc.): hide clap's list and append a localized one
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

    // Localize the label of `[default: x]` as well
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

/// Prints a clap error localized and exits. Errors without a `cli-error-<kind>` key are printed by clap.
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

/// The keys a command needs. Each element lists alternatives, one of which must exist.
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
        return; // its children reuse the keys of the real subcommands
    }
    for s in cmd.get_subcommands() {
        let sp = if path.is_empty() { s.get_name().to_string() } else { format!("{path}-{}", s.get_name()) };
        collect(s, &sp, out);
    }
}
