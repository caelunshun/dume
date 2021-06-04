/// A font weight, indicating how dark it appears.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// A font family in a query, indicating what type of font to use.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum Family {
    /// Any sans serif font.
    SansSerif,
    /// Any serif font.
    Serif,
    /// Only font families with the given name.
    Name(String),
}

impl Default for Family {
    fn default() -> Self {
        Self::SansSerif
    }
}

impl<'a> From<&'a Family> for fontdb::Family<'a> {
    fn from(f: &'a Family) -> Self {
        match f {
            Family::SansSerif => fontdb::Family::SansSerif,
            Family::Serif => fontdb::Family::Serif,
            Family::Name(s) => fontdb::Family::Name(s.as_str()),
        }
    }
}

/// Font style: normal or italic. We do not support
/// oblique fonts.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone)]
pub struct Query {
    pub families: Vec<Family>,
    pub style: Style,
    pub weight: Weight,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            families: vec![Family::default()],
            style: Style::default(),
            weight: Weight::default(),
        }
    }
}

impl Query {
    pub fn with_fontdb_family<R>(&self, f: impl FnOnce(&fontdb::Query) -> R) -> R {
        let query = fontdb::Query {
            families: &self.families.iter().map(From::from).collect::<Vec<_>>(),
            style: self.style.into(),
            weight: self.weight.into(),
            stretch: fontdb::Stretch::default(),
        };
        f(&query)
    }
}
