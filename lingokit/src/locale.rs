//! Choosing the display locale.

use std::ffi::OsString;

use unic_langid::LanguageIdentifier;

/// The requested locale: the first valid value of `cli_lang`, `<env_prefix>_LANG`, `LC_ALL`,
/// `LC_MESSAGES` and `LANG`. Empty when none is set.
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

/// Parses `ja_JP.UTF-8`, `ja-JP`, `ja`, ... into a language identifier. `C` and `POSIX` give `None`.
pub fn parse_posix(v: &str) -> Option<LanguageIdentifier> {
    let v = v.trim().split(['.', '@']).next().unwrap_or("");
    if v.is_empty() || v.eq_ignore_ascii_case("C") || v.eq_ignore_ascii_case("POSIX") {
        return None;
    }
    v.replace('_', "-").parse().ok()
}

/// Finds `--lang <v>` or `--lang=<v>` in the arguments before clap parses them. Stops at `--`.
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
