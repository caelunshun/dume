//! Rich text implementation.

use palette::Srgba;

use crate::font::Query;

/// Some rich text. Implemented as a list of [`TextSection`]s.
#[derive(Debug, Clone)]
pub struct Text {
    sections: Vec<TextSection>,
}

/// A block of text with the same style.
#[derive(Debug, Clone)]
pub struct TextSection {
    text: String,
    style: TextStyle,
}

/// Style of a text section.
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// Text color.
    color: Srgba,
    /// Font size in logical pixels.
    size: f32,
    /// The font to use. Accounts for bold and italics too.
    font: Query,
}
