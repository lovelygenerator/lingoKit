use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use unic_langid::LanguageIdentifier;

use crate::{Locales, LocalizedError, Msg};

/// lingokit 自身の組み込みメッセージ(clap の見出し・エラーなど)。利用側の FTL で上書きできる。
static BUILTIN: &[(&str, &str)] = &[("en", include_str!("../builtin/en/lingokit.ftl")), ("ja", include_str!("../builtin/ja/lingokit.ftl"))];

/// 言語のフォールバックチェーン(例: ja-JP → ja → en)を持つ翻訳器。
///
/// `Send + Sync` なので `Arc<Localizer>` として共有できる。
pub struct Localizer {
    locales: &'static Locales,
    chain: Vec<(LanguageIdentifier, FluentBundle<FluentResource>)>,
}

impl std::fmt::Debug for Localizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let langs: Vec<String> = self.chain.iter().map(|(l, _)| l.to_string()).collect();
        f.debug_struct("Localizer").field("chain", &langs).finish()
    }
}

impl Localizer {
    /// 要求言語(優先度順)から、利用可能な言語と折衝して作る。
    pub fn new(locales: &'static Locales, requested: &[LanguageIdentifier]) -> Self {
        let default: LanguageIdentifier = locales.default_lang.parse().expect("default_lang が不正です");
        let available: Vec<LanguageIdentifier> =
            locales.resources.iter().filter_map(|(l, _)| l.parse().ok()).collect();
        let mut langs: Vec<LanguageIdentifier> =
            negotiate_languages(requested, &available, Some(&default), NegotiationStrategy::Filtering)
                .into_iter()
                .cloned()
                .collect();
        if !langs.contains(&default) {
            langs.push(default);
        }
        let chain = langs.into_iter().map(|l| (l.clone(), build_bundle(locales, &l))).collect();
        Self { locales, chain }
    }

    /// CLI の `--lang` 値(あれば)と環境変数から作る。
    /// 優先順位: `cli_lang` → `<PREFIX>_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG` → 既定言語
    pub fn from_env(locales: &'static Locales, cli_lang: Option<&str>) -> Self {
        Self::new(locales, &crate::locale::resolve(locales.env_prefix, cli_lang))
    }

    /// 単一言語のみ(フォールバックなし)。キー整合性テスト用。
    pub fn only(locales: &'static Locales, lang: &str) -> Self {
        let l: LanguageIdentifier = lang.parse().expect("不正な言語タグ");
        Self { locales, chain: vec![(l.clone(), build_bundle(locales, &l))] }
    }

    pub fn locales(&self) -> &'static Locales {
        self.locales
    }

    /// 実際に使われる第一言語。
    pub fn lang(&self) -> &LanguageIdentifier {
        &self.chain[0].0
    }

    /// 第一言語にキーがあるか(フォールバックしない)。`id.attr` 形式も可。
    pub fn has_own(&self, id: &str) -> bool {
        lookup(&self.chain[0].1, id, None).is_some()
    }

    /// キーを引く。見つからなければ `None`。`id.attr` 形式で属性も引ける。
    pub fn get(&self, id: &str, args: Option<&FluentArgs>) -> Option<String> {
        self.chain.iter().find_map(|(_, b)| lookup(b, id, args))
    }

    /// キーを引く。見つからなければ ID をそのまま返す。
    pub fn fmt(&self, id: &str, args: Option<&FluentArgs>) -> String {
        self.get(id, args).unwrap_or_else(|| id.to_string())
    }

    /// 生成アクセサ(`msg::*`)の結果を文字列にする。
    pub fn msg(&self, m: &Msg<'_>) -> String {
        self.fmt(m.id, m.args.as_ref())
    }

    /// エラーをユーザー向けメッセージにする(source の連鎖は含まない)。
    pub fn error(&self, e: &dyn LocalizedError) -> String {
        self.msg(&e.message())
    }
}

fn build_bundle(locales: &Locales, lang: &LanguageIdentifier) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new_concurrent(vec![lang.clone()]);
    // 端末出力に Unicode 分離記号(U+2068/U+2069)が混ざらないようにする
    bundle.set_use_isolating(false);
    let key = lang.to_string();
    if let Some((_, src)) = BUILTIN.iter().find(|(l, _)| *l == key) {
        let res = FluentResource::try_new((*src).to_string()).expect("組み込み FTL が不正です");
        bundle.add_resource(res).expect("組み込み FTL が不正です");
    }
    if let Some((_, files)) = locales.resources.iter().find(|(l, _)| *l == key) {
        for (name, src) in *files {
            // 構文は build.rs で検証済み
            let res = FluentResource::try_new((*src).to_string())
                .unwrap_or_else(|_| panic!("{key}/{name} のパースに失敗しました"));
            // 利用側の定義で組み込みメッセージを上書きできるようにする
            bundle.add_resource_overriding(res);
        }
    }
    bundle
}

fn lookup(bundle: &FluentBundle<FluentResource>, id: &str, args: Option<&FluentArgs>) -> Option<String> {
    let (mid, attr) = match id.split_once('.') {
        Some((m, a)) => (m, Some(a)),
        None => (id, None),
    };
    let msg = bundle.get_message(mid)?;
    let pattern = match attr {
        Some(a) => msg.get_attribute(a)?.value(),
        None => msg.value()?,
    };
    let mut errors = Vec::new();
    Some(bundle.format_pattern(pattern, args, &mut errors).into_owned())
}
