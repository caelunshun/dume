//! Rich text implementation.

#![allow(clippy::clippy::derive_hash_xor_eq)]

mod layout;
pub mod markup;

use std::hash::Hash;

use palette::Srgba;

use crate::font::{Query, Style, Weight};

pub type FontId = fontdb::ID;

/// Some rich text. Implemented as a list of [`TextSection`]s.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text {
    sections: Vec<TextSection>,
}

impl Text {
    pub fn from_sections(sections: Vec<TextSection>) -> Self {
        Self { sections }
    }

    pub fn sections(&self) -> &[TextSection] {
        &self.sections
    }
}

/// A block of text with the same style.
#[derive(Debug, Clone, PartialEq)]
pub enum TextSection {
    /// Render a string of glyphs.
    Text { text: String, style: TextStyle },
    /// Embed an icon inside text.
    Icon {
        /// Name of the sprite registered in the sprite registry.
        ///
        /// Dume will search both for the sprite called `{name}` and the sprite
        /// called "icon/{name}".
        name: String,
        /// Height of the icon. Matches the size of a glyph with the same size.
        size: f32,
    },
}

impl Eq for TextSection {}

impl Hash for TextSection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TextSection::Text { text, style } => {
                0u8.hash(state);
                text.hash(state);
                style.hash(state);
            }
            TextSection::Icon { name, size } => {
                1u8.hash(state);
                name.hash(state);
                size.to_bits().hash(state);
            }
        };
    }
}

/// Style of a text section.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Text color.
    color: Srgba<u8>,
    /// Font size in logical pixels.
    size: f32,
    /// The font to use. Accounts for bold and italics too.
    font: Query,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Srgba::new(0, 0, 0, u8::MAX),
            size: 12.0,
            font: Query {
                family: "Times New Roman".to_owned(),
                style: Style::Normal,
                weight: Weight::Normal,
            },
        }
    }
}

impl Hash for TextStyle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.color.red.hash(state);
        self.color.green.hash(state);
        self.color.blue.hash(state);
        self.color.alpha.hash(state);
        self.size.to_bits().hash(state);
        self.font.hash(state);
    }
}

impl Eq for TextStyle {}
