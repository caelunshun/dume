use std::borrow::Cow;

use fontdb::Weight;

use crate::Color;

/// The style of a span of text.
///
/// Most parameters are `Option`s. If set to `None`,
/// they default to the default style parameters of the `Text`
/// that contains the span with this style.
/// If set to `Some`, they override the default style.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    weight: Option<Weight>,
    italic: Option<bool>,
    underlined: Option<bool>,
    font_family: Option<FontFamily<'static>>,
    color: Option<Color>,
}

impl Style {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn weight(&self) -> Option<Weight> {
        self.weight
    }

    pub fn is_italic(&self) -> Option<bool> {
        self.italic
    }

    pub fn is_underlined(&self) -> Option<bool> {
        self.underlined
    }

    pub fn font_family(&self) -> Option<&FontFamily> {
        self.font_family.as_ref()
    }

    pub fn color(&self) -> Option<Color> {
        self.color
    }

    pub fn set_weight(&mut self, weight: Weight) {
        self.weight = Some(weight);
    }

    pub fn set_italic(&mut self, italic: bool) {
        self.italic = Some(italic);
    }

    pub fn set_underlined(&mut self, underline: bool) {
        self.underlined = Some(underline);
    }

    pub fn set_font_family(&mut self, font_family: FontFamily<'static>) {
        self.font_family = Some(font_family);
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = Some(color);
    }

    pub fn clear_weight(&mut self) {
        self.weight = None;
    }

    pub fn clear_italic(&mut self) {
        self.italic = None;
    }

    pub fn clear_underlined(&mut self) {
        self.underlined = None;
    }

    pub fn clear_font_family(&mut self) {
        self.font_family = None;
    }

    pub fn clear_color(&mut self) {
        self.color = None;
    }

    /// Resolves the style using the given `Style`
    /// as a set of default values.
    pub(crate) fn resolve_with_defaults<'a>(
        &'a self,
        defaults: &'a Style,
        fallback_font_family: &'a str,
    ) -> ResolvedStyle<'a> {
        ResolvedStyle {
            weight: self.weight.or(defaults.weight).unwrap_or(Weight::NORMAL),
            italic: self.italic.or(defaults.italic).unwrap_or(false),
            underlined: self.underlined.or(defaults.underlined).unwrap_or(false),
            font_family: self
                .font_family
                .as_ref()
                .map(FontFamily::as_ref)
                .or(defaults.font_family.as_ref().map(FontFamily::as_ref))
                .unwrap_or_else(|| FontFamily::named(fallback_font_family)),
            color: self.color.or(defaults.color).unwrap_or(Color::BLACK),
        }
    }
}

/// A "resolved" style with all parameters set.
///
/// Parameters that were set to `None` are changed to a default value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedStyle<'a> {
    weight: Weight,
    italic: bool,
    underlined: bool,
    font_family: FontFamily<'a>,
    color: Color,
}

impl<'a> ResolvedStyle<'a> {
    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn is_italic(&self) -> bool {
        self.italic
    }

    pub fn is_underlined(&self) -> bool {
        self.underlined
    }

    pub fn font_family(&self) -> &FontFamily<'a> {
        &self.font_family
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

/// A reference to a font family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFamily<'a> {
    name: Cow<'a, str>,
    keyed: bool,
}

impl<'a> FontFamily<'a> {
    /// Creates a font family preference that matches
    /// exactly the name of the font.
    pub fn named(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            keyed: false,
        }
    }

    /// Creates a font family preference that matches
    /// a font registered with `Context::create_font_key`.
    ///
    /// Font keying allows you to abstract a font used
    /// in your application (e.g., "font for articles" or "font for menu")
    /// under an opaque key string.
    pub fn keyed(key: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: key.into(),
            keyed: true,
        }
    }

    pub fn is_keyed(&self) -> bool {
        self.keyed
    }

    pub fn name_or_key(&self) -> &str {
        &self.name
    }

    pub fn as_ref<'b>(&'b self) -> FontFamily<'b> {
        FontFamily {
            name: Cow::Borrowed(&self.name),
            keyed: self.keyed,
        }
    }
}
