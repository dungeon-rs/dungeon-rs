#![doc = include_str!("../README.md")]

mod linters;

use proc_macro::TokenStream;

/// An attribute intended to annotate Bevy systems.
/// It automatically annotates the system with several `#[allow]` lints common to Bevy systems for
/// our linting configuration.
///
/// The idea is that we can define all ignored lints for Bevy systems once and update them in one
/// place rather than spread across the entire codebase.
///
/// Currently, this macro applies the following `#[allow]`:
/// * [`clippy::missing_errors_doc`](https://rust-lang.github.io/rust-clippy/master/index.html#missing_errors_doc)
/// * [`clippy::needless_pass_by_value`](https://rust-lang.github.io/rust-clippy/master/index.html#needless_pass_by_value)
#[proc_macro_attribute]
pub fn bevy_system(attr: TokenStream, item: TokenStream) -> TokenStream {
    linters::bevy_system(attr, item)
}
