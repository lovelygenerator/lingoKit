//! ロケール解決。

use std::ffi::OsString;

use unic_langid::LanguageIdentifier;

/// 優先度順の要求言語リストを返す。
/// 優先順位: `cli_lang` → `<prefix>_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG`
pub fn resolve(env_prefix: &str, cli_lang: Option<&str>) -> Vec<LanguageIdentifier> {
    let app_env = format!("{env_prefix}_LANG");
    let env_vals = [app_env.as_str(), "LC_ALL", "LC_MESSAGES", "LANG"].map(|k| std::env::var(k).ok());
    std::iter::once(cli_lang.map(str::to_string))
        .chain(env_vals)
        .flatten()
        .find_map(|v| parse_posix(&v))
        .into_iter()
        .collect()
}

/// `ja_JP.UTF-8` / `ja-JP` / `ja` などを言語タグに正規化する。`C` と `POSIX` は無視する。
pub fn parse_posix(v: &str) -> Option<LanguageIdentifier> {
    let v = v.trim().split(['.', '@']).next().unwrap_or("");
    if v.is_empty() || v.eq_ignore_ascii_case("C") || v.eq_ignore_ascii_case("POSIX") {
        return None;
    }
    v.replace('_', "-").parse().ok()
}

/// clap に渡す前に argv から `--lang <v>` / `--lang=<v>` を拾う。`--` 以降は見ない。
pub fn pre_scan_lang<I, T>(args: I) -> Option<String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut it = args.into_iter().map(Into::into).skip(1);
    while let Some(a) = it.next() {
        let a = a.to_string_lossy().into_owned();
        if a == "--" {
            break;
        }
        if a == "--lang" {
            return it.next().map(|v| v.to_string_lossy().into_owned());
        }
        if let Some(v) = a.strip_prefix("--lang=") {
            return Some(v.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posix_forms() {
        assert_eq!(parse_posix("ja_JP.UTF-8").unwrap().to_string(), "ja-JP");
        assert!(parse_posix("C").is_none());
        assert!(parse_posix("").is_none());
    }

    #[test]
    fn pre_scan() {
        assert_eq!(pre_scan_lang(["app", "--lang", "ja"]), Some("ja".into()));
        assert_eq!(pre_scan_lang(["app", "x", "--lang=en"]), Some("en".into()));
        assert_eq!(pre_scan_lang(["app", "--", "--lang", "ja"]), None);
    }
}
