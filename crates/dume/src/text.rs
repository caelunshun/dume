//! Rich text implementation.

use std::hash::Hash;

use palette::Srgba;
use smallvec::SmallVec;
use smartstring::{LazyCompact, SmartString};

use crate::font::{Query, Style, Weight};

pub mod layout;

/// Some rich text. Implemented as a list of [`TextSection`]s.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text {
    sections: SmallVec<[TextSection; 1]>,
}

impl Text {
    pub fn from_sections(sections: impl IntoIterator<Item = TextSection>) -> Self {
        Self {
            sections: sections.into_iter().collect(),
        }
    }

    pub fn sections(&self) -> &[TextSection] {
        &self.sections
    }

    pub fn to_unstyled_string(&self) -> SmartString<LazyCompact> {
        let mut s = SmartString::new();
        for section in &self.sections {
            if let TextSection::Text { text, .. } = section {
                s.push_str(text);
            }
        }
        s
    }
}

/// A block of text with the same style.
#[derive(Debug, Clone, PartialEq)]
pub enum TextSection {
    /// Render a string of glyphs.
    Text {
        text: SmartString<LazyCompact>,
        style: TextStyle,
    },
    /// Embed an icon inside text.
    Icon {
        /// Name of the texture registered in the context textures.
        ///
        /// Dume will search both for the texture called `{name}` and the texture
        /// called "icon/{name}".
        name: SmartString<LazyCompact>,
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
    pub color: Srgba<u8>,
    /// Font size in logical pixels.
    pub size: f32,
    /// The font to use. Accounts for bold and italics too.
    pub font: Query,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX),
            size: 12.0,
            font: Query {
                family: None,
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
