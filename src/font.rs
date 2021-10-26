//! Font query implementation.
//!
//! Font parsing, shaping, and rendering is handled by the `swash` crate.

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use swash::{CacheKey, FontRef, StringId};

/// A font weight, indicating how dark it appears.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl Default for Weight {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<Weight> for swash::Weight {
    fn from(w: Weight) -> Self {
        use swash::Weight as W;
        match w {
            Weight::Thin => W::THIN,
            Weight::ExtraLight => W::EXTRA_LIGHT,
            Weight::Light => W::LIGHT,
            Weight::Normal => W::NORMAL,
            Weight::Medium => W::MEDIUM,
            Weight::SemiBold => W::SEMI_BOLD,
            Weight::Bold => W::BOLD,
            Weight::ExtraBold => W::EXTRA_BOLD,
            Weight::Black => W::BLACK,
        }
    }
}

/// Font style: normal or italic. We do not support
/// oblique fonts.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Style {
    Normal,
    Italic,
}

impl Default for Style {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<Style> for swash::Style {
    fn from(s: Style) -> Self {
        match s {
            Style::Normal => swash::Style::Normal,
            Style::Italic => swash::Style::Italic,
        }
    }
}

/// A font query. Specifies which fonts can
/// be used in a given context.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Query {
    /// The font family to use. If `None`,
    /// we'll use the default font configured on the `Context`,
    /// or panic if no default font was set.
    pub family: Option<SmartString<LazyCompact>>,
    pub style: Style,
    pub weight: Weight,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            family: None,
            style: Style::Normal,
            weight: Weight::Normal,
        }
    }
}

impl Query {
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }

    pub fn family(mut self, family: &str) -> Self {
        self.family = Some(family.into());
        self
    }

    fn matches(&self, font: &FontRef, default_family: Option<&str>) -> bool {
        let attributes = font.attributes();

        attributes.style() == swash::Style::from(self.style)
            && attributes.weight() == swash::Weight::from(self.weight)
            && font
                .localized_strings()
                .find_by_id(StringId::Family, None)
                .expect("missing font family string")
                .to_string()
                == self.family.as_ref().map(|c| c.as_ref()).unwrap_or_else(|| {
                    default_family.expect(
                        "no default font family was set, but font::Query uses None as the family",
                    )
                })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to parse font as TTF/OTF font data")]
pub struct MalformedFont;

#[derive(Debug, thiserror::Error)]
#[error("no font satisfied the query {0:#?}")]
pub struct MissingFont(Query);

pub(crate) struct Font {
    data: Vec<u8>,
    key: CacheKey,
    offset: u32,
}

impl Font {
    pub fn from_data(data: Vec<u8>) -> Result<Self, MalformedFont> {
        let font = FontRef::from_index(&data, 0).ok_or(MalformedFont)?;
        let FontRef { key, offset, .. } = font;
        Ok(Self { data, key, offset })
    }

    /// The main entrypoint to access font data through `swash`.
    pub fn as_ref(&self) -> FontRef {
        FontRef {
            data: &self.data,
            key: self.key,
            offset: self.offset,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct FontId(usize);

/// The fonts available to a `Context`.
#[derive(Default)]
pub(crate) struct Fonts {
    fonts: Vec<Font>,
    default_family: Option<String>,
}

impl Fonts {
    pub fn add(&mut self, font: Font) -> FontId {
        let id = FontId(self.fonts.len());
        self.fonts.push(font);
        let name = self
            .get(id)
            .localized_strings()
            .find_by_id(StringId::Family, None)
            .expect("missing font family string");
        log::info!("Loaded font '{}'", name);
        id
    }

    pub fn get(&self, id: FontId) -> FontRef {
        self.fonts[id.0].as_ref()
    }

    pub fn set_default_family(&mut self, family: String) {
        self.default_family = Some(family);
    }

    pub fn query(&self, query: &Query) -> Result<FontId, MissingFont> {
        let i = self.fonts.iter().position(|font| {
            query.matches(
                &font.as_ref(),
                self.default_family.as_ref().map(|s| s.as_str()),
            )
        });

        i.map(FontId).ok_or_else(|| MissingFont(query.clone()))
    }
}
