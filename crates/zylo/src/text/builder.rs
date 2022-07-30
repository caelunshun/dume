//! Builder-like API for making rich text.

use fontdb::Weight;

use crate::Color;

use super::{
    span::{Span, Text},
    style::FontFamily,
};

pub trait IntoSpan {
    fn into_span(self) -> Span;
}

impl<'a> IntoSpan for &'a str {
    fn into_span(self) -> Span {
        Span::new(self)
    }
}

impl IntoSpan for String {
    fn into_span(self) -> Span {
        Span::new(self)
    }
}

pub trait BuildText {
    fn weight(self, weight: Weight) -> Text;
    fn clear_weight(self) -> Text;

    fn bold(self) -> Text;
    fn not_bold(self) -> Text;

    fn italic(self) -> Text;
    fn not_italic(self) -> Text;

    fn underlined(self) -> Text;
    fn not_underlined(self) -> Text;

    fn font_family(self, family: FontFamily<'static>) -> Text;
    fn clear_font_family(self) -> Text;

    fn color(self, color: impl Into<Color>) -> Text;
    fn clear_color(self) -> Text;

    fn font_size(self, size: f32) -> Text;
    fn clear_font_size(self) -> Text;

    fn append(self, text: impl Into<Text>) -> Text;
}

impl<T> BuildText for T
where
    T: Into<Span>,
{
    fn weight(self, weight: Weight) -> Text {
        Text::from(self).weight(weight)
    }

    fn clear_weight(self) -> Text {
        Text::from(self).clear_weight()
    }

    fn bold(self) -> Text {
        Text::from(self).bold()
    }

    fn not_bold(self) -> Text {
        Text::from(self).not_bold()
    }

    fn italic(self) -> Text {
        Text::from(self).italic()
    }

    fn not_italic(self) -> Text {
        Text::from(self).not_italic()
    }

    fn underlined(self) -> Text {
        Text::from(self).underlined()
    }

    fn not_underlined(self) -> Text {
        Text::from(self).not_underlined()
    }

    fn font_family(self, family: FontFamily<'static>) -> Text {
        Text::from(self).font_family(family)
    }

    fn clear_font_family(self) -> Text {
        Text::from(self).clear_font_family()
    }

    fn color(self, color: impl Into<Color>) -> Text {
        Text::from(self).color(color)
    }

    fn clear_color(self) -> Text {
        Text::from(self).clear_color()
    }

    fn font_size(self, size: f32) -> Text {
        Text::from(self).font_size(size)
    }

    fn clear_font_size(self) -> Text {
        Text::from(self).clear_font_size()
    }

    fn append(self, text: impl Into<Text>) -> Text {
        Text::from(self).append(text)
    }
}

impl BuildText for Text {
    fn weight(mut self, weight: Weight) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().set_weight(weight));
        self
    }

    fn clear_weight(mut self) -> Text {
        self.spans_mut().for_each(|s| s.style_mut().clear_weight());
        self
    }

    fn bold(self) -> Text {
        self.weight(Weight::BOLD)
    }

    fn not_bold(self) -> Text {
        self.clear_weight()
    }

    fn italic(mut self) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().set_italic(true));
        self
    }

    fn not_italic(mut self) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().set_italic(false));
        self
    }

    fn underlined(mut self) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().set_underlined(true));
        self
    }

    fn not_underlined(mut self) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().set_underlined(false));
        self
    }

    fn font_family(mut self, family: FontFamily<'static>) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().set_font_family(family.clone()));
        self
    }

    fn clear_font_family(mut self) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().clear_font_family());
        self
    }

    fn color(mut self, color: impl Into<Color>) -> Text {
        let color = color.into();
        self.spans_mut()
            .for_each(|s| s.style_mut().set_color(color));
        self
    }

    fn clear_color(mut self) -> Text {
        self.spans_mut().for_each(|s| s.style_mut().clear_color());
        self
    }

    fn font_size(mut self, size: f32) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().set_font_size(size));
        self
    }

    fn clear_font_size(mut self) -> Text {
        self.spans_mut()
            .for_each(|s| s.style_mut().clear_font_size());
        self
    }

    fn append(mut self, text: impl Into<Text>) -> Text {
        self.extend(text.into().into_spans());
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::text::style::Style;

    use super::*;

    #[test]
    fn api() {
        let text = "Goodbye World :("
            .bold()
            .underlined()
            .color(Color::WHITE)
            .append(" (though I never liked it anyway)");
        let mut expected_style = Style::empty();
        expected_style.set_weight(Weight::BOLD);
        expected_style.set_underlined(true);
        expected_style.set_color(Color::WHITE);
        assert_eq!(
            text.spans().collect::<Vec<_>>(),
            vec![
                &Span::with_style("Goodbye World :(", expected_style),
                &Span::new(" (though I never liked it anyway)")
            ]
        );
    }
}
