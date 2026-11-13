use lingokit::{LocalizedError, Localizer, render_error_chain};
use lingokit_demo::{Cli, EmptyError, Error, LOCALES, Server, msg};

#[test]
fn derive_named_fields() {
    let loc = Localizer::only(&LOCALES, "ja");
    let e = Error::NotFound { path: "a.txt".into() };
    assert_eq!(loc.error(&e), "見つかりません: a.txt");
    let e = Error::TooMany { count: 3, _internal: 0 };
    assert_eq!(loc.error(&e), "多すぎます: 3 件");
    let en = Localizer::only(&LOCALES, "en");
    assert_eq!(en.error(&Error::TooMany { count: 1, _internal: 0 }), "Too many: 1 item");
}

#[test]
fn derive_tuple_and_struct() {
    let loc = Localizer::only(&LOCALES, "ja");
    let e = Error::Io(std::io::Error::other("disk"));
    assert_eq!(render_error_chain(&loc, &e), "入出力エラー: disk");
    assert_eq!(loc.error(&EmptyError), "空です");
    assert_eq!(EmptyError.message().id, "err-empty");
}

#[test]
fn typed_accessor_and_attr() {
    let loc = Localizer::only(&LOCALES, "ja");
    assert_eq!(loc.msg(&msg::err_not_found("x")), "見つかりません: x");
    assert_eq!(loc.msg(&msg::err_not_found_hint()), "パスを確認してください。");
    assert_eq!(LOCALES.env_prefix, "DEMO");
}

#[test]
fn builtin_messages_and_override() {
    let loc = Localizer::only(&LOCALES, "ja");
    assert_eq!(loc.fmt("cli-heading-usage", None), "使い方");
    assert_eq!(loc.fmt("cli-about", None), "デモ CLI");
}

#[test]
fn clap_parse_and_keys() {
    lingokit::clap::assert_keys::<Cli>(&LOCALES);
    let (cli, loc) = lingokit::clap::parse_from::<Cli, _, _>(&LOCALES, ["demo", "--lang", "ja", "count", "x"]);
    assert_eq!(loc.lang().to_string(), "ja");
    assert!(matches!(cli.command, lingokit_demo::Commands::Count { .. }));
    let help = lingokit::clap::command::<Cli>(&loc).render_help().to_string();
    assert!(help.contains("使い方") && help.contains("表示言語"), "{help}");
}

#[test]
fn rmcp_keys_and_localize() {
    lingokit::rmcp::assert_keys(&Server::base_router(), &LOCALES);
    let loc = Localizer::only(&LOCALES, "ja");
    let r = lingokit::rmcp::localize(Server::base_router(), &loc);
    let t = &r.map.get("count_words").unwrap().attr;
    assert_eq!(t.description.as_deref(), Some("単語を数える"));
    assert_eq!(t.input_schema["properties"]["path"]["description"], "パス");
    let _ = Server::new(loc);
}
