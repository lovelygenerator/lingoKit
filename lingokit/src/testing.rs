//! Helpers for tests that check the keys an application needs.

use crate::{Locales, Localizer};

/// Checks that the locales have the `required` keys. Each element lists alternatives, one of
/// which must exist.
///
/// Keys missing from the default locale are an `Err`. Keys missing from other locales are
/// printed as warnings, as those fall back to the default locale.
pub fn check_required_keys(locales: &'static Locales, required: &[Vec<String>]) -> Result<(), String> {
    let mut missing_default = Vec::new();
    for (lang, _) in locales.resources {
        let loc = Localizer::only(locales, lang);
        let missing: Vec<String> = required
            .iter()
            .filter(|cands| !cands.iter().any(|k| loc.has_own(k)))
            .map(|cands| cands.join(" | "))
            .collect();
        if missing.is_empty() {
            continue;
        }
        if *lang == locales.default_lang {
            missing_default = missing;
        } else {
            for m in &missing {
                eprintln!("warning: [lingokit] {lang}: missing key `{m}` (falls back to {})", locales.default_lang);
            }
        }
    }
    if missing_default.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "add these keys to the FTL files of the default locale {}:\n  {}",
            locales.default_lang,
            missing_default.join("\n  ")
        ))
    }
}

/// [`check_required_keys`], panicking on `Err`.
///
/// # Panics
///
/// When keys are missing from the default locale.
pub fn assert_required_keys(locales: &'static Locales, required: &[Vec<String>]) {
    if let Err(e) = check_required_keys(locales, required) {
        panic!("{e}");
    }
}
