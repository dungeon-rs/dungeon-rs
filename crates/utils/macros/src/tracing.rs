//! Contains helper macros for reducing boilerplate code with generating tracing spans.
//!
//! For console applications we automatically integrate `tracing-indicatif` in our tracing spans
//! so that progress will automatically be tracked.
//! This introduces some overhead, however, and this module aims to provide macros to eliminate (most)
//! of said overhead.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token, parse_macro_input};

/// Structure to represent field = value pairs
struct Field {
    /// The field identifier
    name: syn::Ident,
    /// The value being assigned to `name`.
    value: Expr,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![=]>()?; // Skip the = token
        let value = input.parse()?;

        Ok(Field { name, value })
    }
}

/// Structure to parse the entire macro input
struct SpanArgs {
    /// The name assigned to the span.
    name: Expr,
    /// All fields passed in the span, can be zero or more fields.
    fields: Vec<Field>,
}

impl Parse for SpanArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let fields = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Punctuated::<Field, Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect()
        } else {
            vec![]
        };

        Ok(SpanArgs { name, fields })
    }
}

/// Wrapper macro that generates a wrapped `bevy::prelude::<level>_span`!
///
/// If a `length` field is passed in, this will automatically setup `tracing-indicatif` integration.
pub fn wrapped_span(level: &str, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as SpanArgs);

    // Create the span function identifier dynamically
    let span_fn = format_ident!("{}_span", level);

    // Find and extract the length field
    let mut length_value = None;
    let mut other_fields = vec![];

    for field in args.fields {
        if field.name == "length" {
            length_value = Some(field.value);
        } else {
            other_fields.push(field);
        }
    }

    // Build the output
    let name = args.name;
    let field_tokens = other_fields.into_iter().map(|f| {
        let Field { name, value, .. } = f;
        quote! { #name = #value }
    });

    if let Some(length) = length_value {
        // Custom logic when length is present
        quote! {
            {
                let span = bevy::prelude::#span_fn!(
                    #name,
                    length = #length,
                    #(#field_tokens),*
                );
                span.pb_set_length(#length);
                if let Ok(template) = ProgressStyle::with_template("{wide_bar} {pos}/{len} {msg}") {
                    span.pb_set_style(&template);
                }

                span
            }
        }
    } else {
        // No length field found
        quote! {
            bevy::prelude::#span_fn!(
                #name,
                #(#field_tokens),*
            )
        }
    }
    .into()
}
