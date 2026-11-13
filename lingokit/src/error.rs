use crate::{Localizer, Msg};

/// An error with a localized message for users.
///
/// Usually implemented with `#[derive(lingokit::LocalizedError)]` and `#[ftl("key")]`.
/// `Display` (e.g. thiserror's `#[error]`) stays for logs; this message is for display.
pub trait LocalizedError: std::error::Error {
    /// The message ID and arguments of this error.
    fn message(&self) -> Msg<'_>;
}

/// Formats `<message>: <cause>: <cause>`. The causes from `source()` use their `Display`.
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
