//! `#[derive(LocalizedError)]`
//!
//! ```ignore
//! #[derive(Debug, thiserror::Error, lingokit::LocalizedError)]
//! pub enum Error {
//!     #[error("not found: {}", path.display())]
//!     #[ftl("err-not-found")]
//!     NotFound { path: PathBuf },
//! }
//! ```
//!
//! - Named fields are passed as Fluent variables of the same name (`$path`).
//! - Fields marked `#[source]`, `#[from]`, `#[backtrace]` or `#[ftl(skip)]`, and fields named
//!   `source` or `backtrace`, are not passed.
//! - Numbers stay numbers (for plural selection), paths use `display()`, other types `Display`.
//! - It is a compile error when the key is not in the FTL files, or when the message uses a
//!   variable that is not a field. This needs `lingokit::include_locales!()` at the crate root;
//!   elsewhere, add `#[ftl(msg = path::to::msg)]` to the type.
//!
//! Re-exported as `lingokit::LocalizedError`; use it through the `lingokit` crate.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::{Attribute, Data, DeriveInput, Fields, LitStr, Path, parse_macro_input};

/// Derives `lingokit::LocalizedError`. See the [crate] documentation.
#[proc_macro_derive(LocalizedError, attributes(ftl))]
pub fn derive_localized_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input).unwrap_or_else(|e| e.to_compile_error()).into()
}

struct FtlAttr {
    key: Option<LitStr>,
    msg: Option<Path>,
    skip: bool,
}

fn parse_ftl(attrs: &[Attribute]) -> syn::Result<FtlAttr> {
    let mut out = FtlAttr { key: None, msg: None, skip: false };
    for attr in attrs.iter().filter(|a| a.path().is_ident("ftl")) {
        // #[ftl("key")]
        if let Ok(lit) = attr.parse_args::<LitStr>() {
            out.key = Some(lit);
            continue;
        }
        // #[ftl(skip)] / #[ftl(msg = path)] / #[ftl(key = "...")]
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                out.skip = true;
            } else if meta.path.is_ident("msg") {
                out.msg = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("key") {
                out.key = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error("unknown ftl attribute (expected \"key\", key = ..., msg = ... or skip)"));
            }
            Ok(())
        })?;
    }
    Ok(out)
}

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "static", "struct", "trait", "true", "type",
    "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

/// Key → identifier, with the same rule as lingokit-build
fn key_ident(key: &LitStr) -> syn::Ident {
    let s = key.value().replace(['-', '.'], "_");
    if KEYWORDS.contains(&s.as_str()) { format_ident!("r#{}", s, span = key.span()) } else { syn::Ident::new(&s, key.span()) }
}

fn is_skipped(field: &syn::Field) -> syn::Result<bool> {
    let thiserror_attr = field.attrs.iter().any(|a| {
        a.path().is_ident("source") || a.path().is_ident("from") || a.path().is_ident("backtrace")
    });
    let named_source = field.ident.as_ref().is_some_and(|i| i == "source" || i == "backtrace");
    Ok(thiserror_attr || named_source || parse_ftl(&field.attrs)?.skip)
}

/// (pattern, body) of one variant or struct
fn arm(fields: &Fields, key: &LitStr, msg: &TokenStream2) -> syn::Result<(TokenStream2, TokenStream2)> {
    let ident = key_ident(key);
    let mut names = Vec::new();
    if let Fields::Named(named) = fields {
        for f in &named.named {
            if !is_skipped(f)? {
                names.push(f.ident.clone().unwrap());
            }
        }
    }
    let pattern = match fields {
        Fields::Named(_) => quote!({ #(#names,)* .. }),
        Fields::Unnamed(_) => quote!((..)),
        Fields::Unit => quote!(),
    };
    let name_strs: Vec<String> = names.iter().map(|n| n.to_string()).collect();
    // Point errors at #[ftl("...")]
    let check = quote_spanned! {key.span()=>
        const _: () = ::lingokit::__private::assert_vars(#msg::__vars::#ident, &[#(#name_strs),*]);
    };
    let body = if names.is_empty() {
        quote!({ #check ::lingokit::Msg::new(#key) })
    } else {
        quote!({
            #check
            let mut args = ::lingokit::FluentArgs::new();
            #( args.set(#name_strs, (&&::lingokit::__private::ArgWrap(#names)).__fk_arg()); )*
            ::lingokit::Msg::with_args(#key, args)
        })
    };
    Ok((pattern, body))
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_g, ty_g, where_c) = input.generics.split_for_impl();
    let top = parse_ftl(&input.attrs)?;
    let msg = match &top.msg {
        Some(p) => quote!(#p),
        None => quote!(crate::msg),
    };

    let body = match &input.data {
        Data::Enum(e) => {
            let mut arms = Vec::new();
            for v in &e.variants {
                let key = parse_ftl(&v.attrs)?.key.ok_or_else(|| {
                    syn::Error::new_spanned(&v.ident, "add #[ftl(\"key\")] to every variant")
                })?;
                let (pat, body) = arm(&v.fields, &key, &msg)?;
                let vi = &v.ident;
                arms.push(quote!(Self::#vi #pat => #body));
            }
            quote!(match self { #(#arms,)* })
        }
        Data::Struct(s) => {
            let key = top
                .key
                .clone()
                .ok_or_else(|| syn::Error::new_spanned(name, "add #[ftl(\"key\")] to the struct"))?;
            let (pat, body) = arm(&s.fields, &key, &msg)?;
            quote!({ let Self #pat = self; #body })
        }
        Data::Union(_) => return Err(syn::Error::new_spanned(name, "unions are not supported")),
    };

    Ok(quote! {
        impl #impl_g ::lingokit::LocalizedError for #name #ty_g #where_c {
            #[allow(unused_imports, clippy::needless_borrow)]
            fn message(&self) -> ::lingokit::Msg<'_> {
                use ::lingokit::__private::{ViaDisplay as _, ViaSpecial as _};
                #body
            }
        }
    })
}
