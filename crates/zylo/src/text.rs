//! Text layout and rendering implementation.
//!
//! # Rich text
//! `zylo` supports rendering rich text. The `Text` struct contains
//! a list of `TextSpans`, where each span has its own style.
//! An idiomatic "text builder" API is included to style strings
//! using methods and extension traits.
//!
//! # Limitations
//! Proper support for bidirectional text or text with different
//! line breaking rules is not yet implemented. Some progress was
//! made on more complex text support in older versions of this library,
//! but I don't have the time to improve that. Contributions are of course welcome.
//!
//! The linebreaker will break lines at word boundaries (only spaces are detected)
//! and, optionally, at hyphenization points. It will not work correctly
//! for text in languages with different word conventions.
//!
//! Right-to-left text is not supported.
//!
//! However, ligatures and kerning should work correctly thanks to `rustybuzz`.

pub mod builder;
pub mod font;
pub mod layout;
pub mod span;
pub mod style;

pub use fontdb::Weight;

use rustybuzz::UnicodeBuffer;

use self::font::FontStore;

/// Context for rendering and laying out text.
pub struct TextContext {
    fonts: FontStore,
    cache: Cache,
    fallback_font_family: String,
}

impl TextContext {
    fn fonts(&self) -> &FontStore {
        &self.fonts
    }

    fn fallback_font_family(&self) -> &str {
        &self.fallback_font_family
    }
}

/// Caches some heap-allocated types for reuse.
struct Cache {
    unicode_buffer: UnicodeBuffer,
}

fn points_to_pixels(points: f32) -> f32 {
    points * (16. / 12.)
}
