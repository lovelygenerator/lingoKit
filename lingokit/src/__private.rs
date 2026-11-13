//! Internal API used by the code that `#[derive(LocalizedError)]` generates. Not public API.

use std::fmt::Display;
use std::path::{Path, PathBuf};

use fluent_bundle::FluentValue;

/// Converts a field to a Fluent argument (autoref specialization):
/// `(&&ArgWrap(&x)).__fk_arg()` keeps numbers as numbers, formats paths with `display()`, and
/// uses `Display` for everything else.
pub struct ArgWrap<'a, T: ?Sized>(pub &'a T);

/// Numbers and paths.
pub trait ViaSpecial {
    /// The Fluent value.
    fn __fk_arg(&self) -> FluentValue<'static>;
}

/// Everything that implements `Display`.
pub trait ViaDisplay {
    /// The Fluent value.
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

/// Checks at compile time that every variable of a message is a field.
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
            panic!("the FTL message uses a variable that is not a field of this variant or struct (check the #[ftl] key and the field names)");
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
