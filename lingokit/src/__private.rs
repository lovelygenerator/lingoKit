//! derive マクロが生成するコードから使う内部 API。直接使わないこと。

use std::fmt::Display;
use std::path::{Path, PathBuf};

use fluent_bundle::FluentValue;

/// フィールド値を Fluent の引数に変換するためのラッパー(autoref specialization)。
///
/// `(&&ArgWrap(&x)).__fk_arg()` で、数値・パスは専用の変換、それ以外は `Display` を使う。
pub struct ArgWrap<'a, T: ?Sized>(pub &'a T);

pub trait ViaSpecial {
    fn __fk_arg(&self) -> FluentValue<'static>;
}

pub trait ViaDisplay {
    fn __fk_arg(&self) -> FluentValue<'static>;
}

macro_rules! num {
    ($($t:ty),*) => {$(
        impl ViaSpecial for &ArgWrap<'_, $t> {
            fn __fk_arg(&self) -> FluentValue<'static> { FluentValue::from(*self.0) }
        }
    )*};
}
num!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

impl ViaSpecial for &ArgWrap<'_, Path> {
    fn __fk_arg(&self) -> FluentValue<'static> {
        FluentValue::from(self.0.display().to_string())
    }
}

impl ViaSpecial for &ArgWrap<'_, PathBuf> {
    fn __fk_arg(&self) -> FluentValue<'static> {
        FluentValue::from(self.0.display().to_string())
    }
}

impl<T: Display + ?Sized> ViaDisplay for ArgWrap<'_, T> {
    fn __fk_arg(&self) -> FluentValue<'static> {
        FluentValue::from(self.0.to_string())
    }
}

/// FTL のメッセージが使う変数がすべてフィールドとして渡されているかをコンパイル時に検査する。
pub const fn assert_vars(vars: &[&str], fields: &[&str]) {
    let mut i = 0;
    while i < vars.len() {
        let mut found = false;
        let mut j = 0;
        while j < fields.len() {
            if str_eq(vars[i], fields[j]) {
                found = true;
            }
            j += 1;
        }
        if !found {
            panic!("FTL のメッセージが使う変数に対応するフィールドがありません(#[ftl] のキーとフィールド名を確認してください)");
        }
        i += 1;
    }
}

const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}
