/// A font weight, indicating how dark it appears.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
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

impl From<Weight> for fontdb::Weight {
    fn from(w: Weight) -> Self {
        use fontdb::Weight as W;
        match w {
            Weight::Thin => W::THIN,
            Weight::ExtraLight => W::EXTRA_LIGHT,
            Weight::Light => W::LIGHT,
            Weight::Normal => W::NORMAL,
            Weight::Medium => W::MEDIUM,
            Weight::SemiBold => W::SEMIBOLD,
            Weight::Bold => W::BOLD,
            Weight::ExtraBold => W::EXTRA_BOLD,
            Weight::Black => W::BLACK,
        }
    }
}

/// Font style: normal or italic. We do not support
/// oblique fonts.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Style {
    Normal,
    Italic,
}

impl Default for Style {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<Style> for fontdb::Style {
    fn from(s: Style) -> Self {
        match s {
            Style::Normal => fontdb::Style::Normal,
            Style::Italic => fontdb::Style::Italic,
        }
    }
}

/// A font query. Specifies which fonts can
/// be used in a given context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Query {
    pub family: String,
    pub style: Style,
    pub weight: Weight,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            family: "Merriweather".to_owned(),
            style: Style::default(),
            weight: Weight::default(),
        }
    }
}

impl Query {
    pub fn with_fontdb_family<R>(&self, f: impl FnOnce(&fontdb::Query) -> R) -> R {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(&self.family)],
            style: self.style.into(),
            weight: self.weight.into(),
            stretch: fontdb::Stretch::default(),
        };
        f(&query)
    }
}

pub struct Font<'a> {
    pub hb: rustybuzz::Face<'a>,
    pub ttf: ttf_parser::Face<'a>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Option<Self> {
        Some(Self {
            hb: rustybuzz::Face::from_slice(data, 0)?,
            ttf: ttf_parser::Face::from_slice(data, 0).ok()?,
        })
    }
}
