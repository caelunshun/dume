//! Parser for a markup text format. Includes
//! a procedural macro that expands to a `Text`
//! expression.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
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

    let fmt_args: Vec<_> = input
        .fmt_args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let ident = identnumber(i);
            quote! { let #ident = #arg; }
        })
        .collect();

    let result = text.to_rust_code(
        &(0..input.fmt_args.len())
            .map(identnumber)
            .collect::<Vec<_>>(),
    );
    let result = quote! {
        {
            #(#fmt_args)*
            #result
        }
    };
    result.into()
}

fn identnumber(n: usize) -> Ident {
    Ident::new(&format!("arg{}", n), Span::call_site())
}
