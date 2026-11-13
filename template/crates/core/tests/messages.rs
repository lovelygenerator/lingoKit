use core_lib::{Error, LOCALES, msg};
use lingokit::Localizer;

#[test]
fn error_messages() {
    let ja = Localizer::only(&LOCALES, "ja");
    let e = Error::NotFound { path: "a.txt".into() };
    assert_eq!(ja.error(&e), "ファイルが見つかりません: a.txt");

    let en = Localizer::only(&LOCALES, "en");
    assert_eq!(en.msg(&msg::result_word_count(1, "a.txt")), "a.txt contains 1 word.");
}
