//! Macros that automatically apply or silence lints.
//! The goal is to minimise the amount of lint configuration littered through the codebase.
#![allow(clippy::missing_docs_in_private_items)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, ItemFn, parse_macro_input};

pub fn bevy_system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a function AST
    let mut input = parse_macro_input!(item as ItemFn);

    // Be careful when adding new lints to this list!
    // All systems in the application will have these `allow`s applied, silencing lints.
    // Ensure that the lints are *globally* applicable.
    let allow_attribute: Attribute = syn::parse_quote! {
        #[allow(
            clippy::missing_errors_doc,
            clippy::needless_pass_by_value
        )]
    };

    input.attrs.splice(0..0, std::iter::once(allow_attribute));
    TokenStream::from(quote!(#input))
}
