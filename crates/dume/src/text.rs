//! Rich text implementation.

use palette::Srgba;
use smallvec::SmallVec;
use smartstring::{LazyCompact, SmartString};

use crate::font::{Query, Style, Weight};

pub mod layout;

pub const DEFAULT_SIZE: f32 = 12.;

pub fn default_color() -> Srgba<u8> {
    Srgba::new(0, 0, 0, u8::MAX)
}

/// Some rich text. Implemented as a list of [`TextSection`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    sections: SmallVec<[TextSection; 1]>,
}

impl Text {
    pub fn from_sections(sections: impl IntoIterator<Item = TextSection>) -> Self {
        Self {
            sections: sections.into_iter().collect(),
        }
    }

    pub fn extend(&mut self, other: Text) {
        self.sections.extend(other.sections);
    }

    pub fn sections(&self) -> &[TextSection] {
        &self.sections
    }

    pub fn set_default_size(&mut self, size: f32) {
        self.for_each_style(|s| {
            if s.size.is_none() {
                s.size = Some(size);
            }
        });
    }

    pub fn set_default_color(&mut self, color: Srgba<u8>) {
        self.for_each_style(|s| {
            if s.color.is_none() {
                s.color = Some(color);
            }
        });
    }

    pub fn set_default_font_family(&mut self, family: SmartString<LazyCompact>) {
        self.for_each_style(|s| {
            if s.font.family.is_none() {
                s.font.family = Some(family.clone());
            }
        });
    }

    fn for_each_style(&mut self, mut f: impl FnMut(&mut TextStyle)) {
        for section in &mut self.sections {
            if let TextSection::Text { style, .. } = section {
                f(style);
            }
        }
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

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text::from(s.as_str())
    }
}

impl<'a> From<&'a str> for Text {
    fn from(s: &'a str) -> Self {
        Text::from_sections([TextSection::Text {
            text: s.into(),
            style: Default::default(),
        }])
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

/// Style of a text section.
///
/// Optional fields will use a default value if set to `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Text color.
    pub color: Option<Srgba<u8>>,
    /// Font size in logical pixels.
    pub size: Option<f32>,
    /// The font to use. Accounts for bold and italics too.
    pub font: Query,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: None,
            size: None,
            font: Query {
                family: None,
                style: Style::Normal,
                weight: Weight::Normal,
            },
        }
    }
}

impl Eq for TextStyle {}

impl AsRef<Text> for Text {
    fn as_ref(&self) -> &Text {
        self
    }
}
