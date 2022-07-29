use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

#[derive(Debug, Default)]
pub struct Text {
    pub sections: Vec<TextSection>,
}

#[derive(Debug)]
pub enum TextSection {
    Text { chunk: TextChunk, style: TextStyle },
    Icon { texture: String, size: f32 },
}

#[derive(Debug)]
pub enum TextChunk {
    Literal(String),
    FormatDisplay { fmt_index: usize },
    FormatDebug { fmt_index: usize },
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: Option<Color>,
    pub size: Option<f32>,
    pub bold: bool,
    pub light: bool,
    pub italic: bool,
    pub font: Option<String>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: None,
            size: None,
            bold: false,
            light: false,
            italic: false,
            font: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Color {
    Constant([u8; 4]),
    FmtArg(usize),
}

impl Text {
    pub fn to_rust_code(&self, fmt_args: &[Expr]) -> TokenStream {
        let mut sections = Vec::new();

        for section in &self.sections {
            let t = match section {
                TextSection::Text { chunk, style } => {
                    let text = match chunk {
                        TextChunk::Literal(s) => quote! { #s.into() },
                        TextChunk::FormatDisplay { fmt_index } => {
                            let arg = &fmt_args[*fmt_index];
                            quote! {{
                                let mut s = ::dume::SmartString::new();
                                std::fmt::Write::write_fmt(&mut s, format_args!("{}", #arg)).ok();
                                s
                            }}
                        }
                        TextChunk::FormatDebug { fmt_index } => {
                            let arg = &fmt_args[*fmt_index];
                            quote! {{
                                let mut s = ::dume::SmartString::new();
                                std::fmt::Write::write_fmt(&mut s, format_args!("{:?}", #arg)).ok();
                                s
                            }}
                        }
                    };

                    let TextStyle {
                        color,
                        size,
                        bold,
                        light,

                        italic,
                        font,
                    } = style;

                    let color = match color {
                        Some(Color::Constant([r, g, b, a])) => {
                            quote! { Some(::dume::Srgba::new(#r, #g, #b, #a)) }
                        }
                        Some(Color::FmtArg(arg)) => {
                            let arg = &fmt_args[*arg];
                            quote! {
                                Some(#arg)
                            }
                        }
                        None => quote! { None },
                    };

                    let size = if let Some(size) = size {
                        quote! { Some(#size) }
                    } else {
                        quote! { None }
                    };
                    let weight = if *bold {
                        quote! { ::dume::Weight::Bold }
                    } else if *light {
                        quote! { ::dume::Weight::Light }
                    } else {
                        quote! { ::dume::Weight::Normal }
                    };
                    let style = if *italic {
                        quote! { ::dume::Style::Italic }
                    } else {
                        quote! { ::dume::Style::Normal }
                    };

                    let family = if let Some(f) = font {
                        quote! { Some(#f.into()) }
                    } else {
                        quote! { None }
                    };

                    quote! {
                        ::dume::TextSection::Text {
                            text: #text,
                            style: ::dume::TextStyle {
                                color: #color,
                                size: #size,
                                font: ::dume::font::Query {
                                    family: #family,
                                    weight: #weight,
                                    style: #style,
                                }
                            }
                        }
                    }
                }
                TextSection::Icon { texture, size } => quote! {
                    ::dume::TextSection::Icon {
                        name: #texture.into(),
                        size: #size,
                    }
                },
            };
            sections.push(t);
        }

        quote! {
            [
                #(#sections,)*
            ]
        }
    }
}
