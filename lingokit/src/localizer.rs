use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use unic_langid::LanguageIdentifier;

use crate::{Locales, LocalizedError, Msg};

/// Built-in messages (clap headings, errors, ...). The application's FTL files can override them.
static BUILTIN: &[(&str, &str)] = &[("en", include_str!("../builtin/en/lingokit.ftl")), ("ja", include_str!("../builtin/ja/lingokit.ftl"))];

/// Formats messages in the chosen locale, falling back along a chain (e.g. `ja-JP` → `ja` → `en`).
///
/// `Localizer` is `Send + Sync`, so it can be shared as `Arc<Localizer>`.
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
    /// Negotiates the requested locales (in order of preference) with the available ones.
    pub fn new(locales: &'static Locales, requested: &[LanguageIdentifier]) -> Self {
        let default: LanguageIdentifier = locales.default_lang.parse().expect("invalid default_lang");
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

    /// Chooses the locale from `cli_lang` (a `--lang` value), then `<PREFIX>_LANG`, `LC_ALL`,
    /// `LC_MESSAGES`, `LANG`, then the default locale.
    pub fn from_env(locales: &'static Locales, cli_lang: Option<&str>) -> Self {
        Self::new(locales, &crate::locale::resolve(locales.env_prefix, cli_lang))
    }

    /// A localizer for `lang` only, without fallback. Used to test which keys a locale has.
    pub fn only(locales: &'static Locales, lang: &str) -> Self {
        let l: LanguageIdentifier = lang.parse().expect("invalid locale");
        Self { locales, chain: vec![(l.clone(), build_bundle(locales, &l))] }
    }

    /// The translations this localizer uses.
    pub fn locales(&self) -> &'static Locales {
        self.locales
    }

    /// The first locale of the chain.
    pub fn lang(&self) -> &LanguageIdentifier {
        &self.chain[0].0
    }

    /// Whether the first locale has `id` (`id` or `id.attr`), without fallback.
    pub fn has_own(&self, id: &str) -> bool {
        lookup(&self.chain[0].1, id, None).is_some()
    }

    /// Formats `id` (`id` or `id.attr`), or `None` when no locale of the chain has it.
    pub fn get(&self, id: &str, args: Option<&FluentArgs>) -> Option<String> {
        self.chain.iter().find_map(|(_, b)| lookup(b, id, args))
    }

    /// Formats `id`, or returns `id` itself when it is missing.
    pub fn fmt(&self, id: &str, args: Option<&FluentArgs>) -> String {
        self.get(id, args).unwrap_or_else(|| id.to_string())
    }

    /// Formats a [`Msg`], such as one returned by the generated `msg` functions.
    pub fn msg(&self, m: &Msg<'_>) -> String {
        self.fmt(m.id, m.args.as_ref())
    }

    /// Formats the message of an error (without its `source()` chain; see [`crate::render_error_chain`]).
    pub fn error(&self, e: &dyn LocalizedError) -> String {
        self.msg(&e.message())
    }
}

fn build_bundle(locales: &Locales, lang: &LanguageIdentifier) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new_concurrent(vec![lang.clone()]);
    // Keep Unicode isolation marks (U+2068/U+2069) out of terminal output
    bundle.set_use_isolating(false);
    let key = lang.to_string();
    if let Some((_, src)) = BUILTIN.iter().find(|(l, _)| *l == key) {
        let res = FluentResource::try_new((*src).to_string()).expect("invalid built-in FTL");
        bundle.add_resource(res).expect("invalid built-in FTL");
    }
    if let Some((_, files)) = locales.resources.iter().find(|(l, _)| *l == key) {
        for (name, src) in *files {
            // Already checked by lingokit-build
            let res = FluentResource::try_new((*src).to_string())
                .unwrap_or_else(|_| panic!("failed to parse {key}/{name}"));
            // Let the application override built-in messages
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
