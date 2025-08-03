#![doc = include_str!("../README.md")]

mod linters;
mod tracing;

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

/// A thin wrapper around `bevy::prelude::trace_span!`
///
/// If an argument named `length` is passed in, this will be used as the length for `tracing-indicatif`
/// integration.
#[proc_macro]
pub fn trace_span(item: TokenStream) -> TokenStream {
    tracing::wrapped_span("trace", item)
}

/// A thin wrapper around `bevy::prelude::trace_span!`
///
/// If an argument named `length` is passed in, this will be used as the length for `tracing-indicatif`
/// integration.
#[proc_macro]
pub fn debug_span(item: TokenStream) -> TokenStream {
    tracing::wrapped_span("debug", item)
}

/// A thin wrapper around `bevy::prelude::info_span!`
///
/// If an argument named `length` is passed in, this will be used as the length for `tracing-indicatif`
/// integration.
#[proc_macro]
pub fn info_span(item: TokenStream) -> TokenStream {
    tracing::wrapped_span("info", item)
}

/// A thin wrapper around `bevy::prelude::warn_span!`
///
/// If an argument named `length` is passed in, this will be used as the length for `tracing-indicatif`
/// integration.
#[proc_macro]
pub fn warn_span(item: TokenStream) -> TokenStream {
    tracing::wrapped_span("warn", item)
}

/// A thin wrapper around `bevy::prelude::error_span!`
///
/// If an argument named `length` is passed in, this will be used as the length for `tracing-indicatif`
/// integration.
#[proc_macro]
pub fn error_span(item: TokenStream) -> TokenStream {
    tracing::wrapped_span("error", item)
}
