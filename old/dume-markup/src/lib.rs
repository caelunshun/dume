//! Parser for a markup text format. Includes
//! a procedural macro that expands to a `Text`
//! expression.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, Expr, LitStr, Token};

mod lexer;
mod output;
mod parser;

struct Input {
    markup: LitStr,
    fmt_args: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let markup = input.parse()?;

        let fmt_args = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            Punctuated::parse_terminated(input)?
        } else {
            Punctuated::default()
        };

        Ok(Self { markup, fmt_args })
    }
}

#[proc_macro]
pub fn markup(input: TokenStream) -> TokenStream {
    let input: Input = syn::parse_macro_input!(input);

    let text = parser::parse(&input.markup.value()).expect("failed to parse markup");

    let result = text.to_rust_code(&input.fmt_args.into_iter().collect::<Vec<_>>());
    let result = quote! {
        {
            #result
        }
    };
    result.into()
}
