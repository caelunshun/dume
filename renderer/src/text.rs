//! Rich text implementation.

mod markup;

use palette::Srgba;

use crate::font::{Query, Style, Weight};

/// Some rich text. Implemented as a list of [`TextSection`]s.
#[derive(Debug, Clone, PartialEq)]
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

/// Style of a text section.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Text color.
    color: Srgba,
    /// Font size in logical pixels.
    size: f32,
    /// The font to use. Accounts for bold and italics too.
    font: Query,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Srgba::new(0.0, 0.0, 0.0, 1.0),
            size: 12.0,
            font: Query {
                family: "Times New Roman".to_owned(),
                style: Style::Normal,
                weight: Weight::Normal,
            },
        }
    }
}
