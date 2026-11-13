use crate::{Localizer, Msg};

/// ユーザー向けの翻訳メッセージを持つエラー。
///
/// 通常は `#[derive(lingokit::LocalizedError)]` と `#[ftl("key")]` で実装する。
/// `Display`(thiserror の `#[error]`)は英語のログ用、こちらは表示用。
pub trait LocalizedError: std::error::Error {
    fn message(&self) -> Msg<'_>;
}

/// エラーと `source()` の連鎖を「メッセージ: 原因: 原因」の形で表示する。
/// 連鎖の部分は `Display`(英語)のまま。
pub fn render_error_chain(loc: &Localizer, e: &dyn LocalizedError) -> String {
    let mut out = loc.error(e);
    let mut cur = e.source();
    while let Some(src) = cur {
        out.push_str(": ");
        out.push_str(&src.to_string());
        cur = src.source();
    }
    out
}
