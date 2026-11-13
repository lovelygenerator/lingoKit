//! キー整合性テスト用のヘルパー。

use crate::{Locales, Localizer};

/// 必須キーを検査する。`required` の各要素は「どれか1つがあればよい」候補。
///
/// - 既定言語での欠落 → `Err`
/// - それ以外の言語での欠落 → 警告を表示するだけ(既定言語にフォールバックする)
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
                eprintln!("warning: [lingokit] {lang}: キー `{m}` がありません({} にフォールバック)", locales.default_lang);
            }
        }
    }
    if missing_default.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "既定言語 {} の FTL に次のキーを追加してください:\n  {}",
            locales.default_lang,
            missing_default.join("\n  ")
        ))
    }
}

/// `check_required_keys` が `Err` なら panic する。
pub fn assert_required_keys(locales: &'static Locales, required: &[Vec<String>]) {
    if let Err(e) = check_required_keys(locales, required) {
        panic!("{e}");
    }
}
