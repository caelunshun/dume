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

pub mod font;
pub mod style;
pub mod span;
pub mod builder;

pub use fontdb::Weight;
