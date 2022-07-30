use std::{
    iter::{self, Once},
    mem, slice, vec,
};

use super::style::Style;

/// A span of styled text.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Span {
    text: String,
    style: Style,
}

impl Span {
    pub fn new(text: impl Into<String>) -> Self {
        Self::with_style(text, Style::empty())
    }

    pub fn with_style(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }
}

impl<T> From<T> for Span
where
    T: Into<String>,
{
    fn from(val: T) -> Self {
        Span::new(val)
    }
}

/// Some rich, styled text.
///
/// Consists of a list of `Span`s where each span
/// can have a unique style.
///
/// For the common case where there is only one span,
/// this struct will avoid a heap allocation.
///
/// # Default styles
/// `Text` stores a "default style." Any unset style
/// properties on its spans will default to the default
/// style's values (and, if the default style value is also unset,
/// then a hardcoded default).
///
/// This functionality exists to serve e.g. UI frameworks,
/// where a CSS-like styling language might set the size of
/// some text (making it the default value) but then the rich
/// text provided to the UI overrides that size.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    inner: Inner,
    default_style: Style,
}

impl Text {
    /// Creates a new `Text` from just one span.
    pub fn simple(span: impl Into<Span>) -> Self {
        Self::new(iter::once(span.into()))
    }

    /// Creates a new `Text` from a list of spans.
    pub fn new<I>(spans: I) -> Self
    where
        I: IntoIterator<Item = Span>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut spans = spans.into_iter();

        let inner = match spans.len() {
            1 => Inner::One(spans.next().unwrap()),
            _ => Inner::Many(spans.collect()),
        };

        Self {
            inner,
            default_style: Style::empty(),
        }
    }

    /// Pushes a span onto the text.
    pub fn push(&mut self, span: Span) {
        match &mut self.inner {
            Inner::One(one) => {
                let one = mem::take(one);
                self.inner = Inner::Many(Vec::with_capacity(2));
                match &mut self.inner {
                    Inner::Many(vec) => vec.extend([one, span]),
                    _ => unreachable!(),
                }
            }
            Inner::Many(vec) if vec.is_empty() => self.inner = Inner::One(span),
            Inner::Many(vec) => vec.push(span),
        }
    }

    /// Extends the text from an iterator of spans.
    pub fn extend(&mut self, iter: impl IntoIterator<Item = Span>) {
        for span in iter {
            self.push(span);
        }
    }

    /// Gets an iterator over the text's spans.
    pub fn spans(&self) -> impl Iterator<Item = &Span> {
        match &self.inner {
            Inner::One(span) => SpanIter::One(iter::once(span)),
            Inner::Many(spans) => SpanIter::Many(spans.iter()),
        }
    }

    /// Mutably gets an iterator over the text's spans.
    pub fn spans_mut(&mut self) -> impl Iterator<Item = &mut Span> {
        match &mut self.inner {
            Inner::One(span) => SpanIterMut::One(iter::once(span)),
            Inner::Many(spans) => SpanIterMut::Many(spans.iter_mut()),
        }
    }

    /// Turns the `Text` into an iterator over its spans.
    pub fn into_spans(self) -> impl Iterator<Item = Span> {
        match self.inner {
            Inner::One(one) => IntoSpanIter::One(iter::once(one)),
            Inner::Many(spans) => IntoSpanIter::Many(spans.into_iter()),
        }
    }

    /// Gets the `i`th span in the text.
    pub fn span(&self, i: usize) -> Option<&Span> {
        match &self.inner {
            Inner::One(one) => (i == 0).then_some(one),
            Inner::Many(many) => many.get(i),
        }
    }

    /// Mutably gets the `i`th span in the text.
    pub fn span_mut(&mut self, i: usize) -> Option<&mut Span> {
        match &mut self.inner {
            Inner::One(one) => (i == 0).then_some(one),
            Inner::Many(many) => many.get_mut(i),
        }
    }

    /// Gets the default style for spans in the text.
    pub fn default_style(&self) -> &Style {
        &self.default_style
    }

    /// Mutably gets the default style for spans in the text.
    pub fn default_style_mut(&mut self) -> &mut Style {
        &mut self.default_style
    }

    /// Sets the default style for spans in the text.
    pub fn set_default_style(&mut self, style: Style) {
        self.default_style = style;
    }
}

impl<T> From<T> for Text
where
    T: Into<Span>,
{
    fn from(span: T) -> Self {
        Text::simple(span)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Inner {
    /// Just one text span.
    One(Span),
    /// Many text spans.
    Many(Vec<Span>),
}

enum SpanIter<'a> {
    One(Once<&'a Span>),
    Many(slice::Iter<'a, Span>),
}

impl<'a> Iterator for SpanIter<'a> {
    type Item = &'a Span;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SpanIter::One(one) => one.next(),
            SpanIter::Many(many) => many.next(),
        }
    }
}

enum SpanIterMut<'a> {
    One(Once<&'a mut Span>),
    Many(slice::IterMut<'a, Span>),
}

impl<'a> Iterator for SpanIterMut<'a> {
    type Item = &'a mut Span;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SpanIterMut::One(one) => one.next(),
            SpanIterMut::Many(many) => many.next(),
        }
    }
}

enum IntoSpanIter {
    One(Once<Span>),
    Many(vec::IntoIter<Span>),
}

impl Iterator for IntoSpanIter {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoSpanIter::One(one) => one.next(),
            IntoSpanIter::Many(many) => many.next(),
        }
    }
}
