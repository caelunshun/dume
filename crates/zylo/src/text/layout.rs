use std::mem;

use glam::{vec2, Vec2};
use owned_ttf_parser::{AsFaceRef, GlyphId};
use rustybuzz::Face;

use crate::{Span, Style, Text};

use super::{font::FontStore, points_to_pixels, style::ResolvedStyle, TextContext};

const DEFAULT_BOUNDS: Vec2 = Vec2::splat(f32::INFINITY);

/// A galley represents a blob of [`Text`] that has been shaped
/// and prepared for rendering.
///
/// Call [`lay_out`] to fit the text into a bounding box and to perform additional
/// layout operations like centering, baseline alignment, etc.
#[derive(Debug, Clone)]
pub struct TextGalley {
    span_styles: Vec<ResolvedStyle>,

    items: Vec<Item>,

    /// Precomputed values
    size: Vec2,
}

impl TextGalley {
    pub(crate) fn new(context: &mut TextContext, text: &Text) -> Self {
        let span_styles = resolve_span_styles(context, text);
        let items = shape_to_items(context, text, &span_styles);
        let size = compute_size(&mut context.fonts, &items, &span_styles);

        let mut galley = Self {
            span_styles,
            items,
            size,
        };
        galley.lay_out(context, DEFAULT_BOUNDS);
        galley
    }

    pub(crate) fn lay_out(&mut self, context: &mut TextContext, _max_size: Vec2) {
        let mut cursor = 0.;
        for item in &mut self.items {
            if let Item::Glyph(glyph) = item {
                glyph.pos.x = cursor;
                cursor += glyph.advance;
            }
        }
        self.size = compute_size(&mut context.fonts, &self.items, &self.span_styles);
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn span_styles(&self) -> &[ResolvedStyle] {
        &self.span_styles
    }
}

fn resolve_span_styles(context: &TextContext, text: &Text) -> Vec<ResolvedStyle> {
    text.spans()
        .map(|span| {
            span.style().resolve_with_defaults(
                text.default_style(),
                context.fonts(),
                context.fallback_font_family(),
            )
        })
        .collect()
}

fn shape_to_items(
    context: &mut TextContext,
    text: &Text,
    span_styles: &[ResolvedStyle],
) -> Vec<Item> {
    let mut items = Vec::with_capacity(text.spans().map(|s| s.text().len()).sum());

    let mut base_index = 0;
    for (i, (span, style)) in text.spans().zip(span_styles.iter()).enumerate() {
        shape_span_to_items(
            context,
            base_index,
            i.try_into().unwrap(),
            span,
            style,
            &mut items,
        );
        base_index += u32::try_from(span.text().len()).unwrap();
    }

    items
}

fn shape_span_to_items(
    context: &mut TextContext,
    base_index: u32,
    span_index: u32,
    span: &Span,
    style: &ResolvedStyle,
    items: &mut Vec<Item>,
) {
    let mut unicode_buffer = mem::take(&mut context.cache.unicode_buffer);

    unicode_buffer.push_str(span.text());
    // TODO properly handle text itemization with language/script/BiDi spans.
    // For now we assume the whole span has the same properties.
    unicode_buffer.guess_segment_properties();

    let face = context.fonts.get(style.font());
    let rb_face = rustybuzz::Face::from_face(face.clone())
        .expect("invalid font for shaping (units_per_em must be set)");

    let glyph_buffer = rustybuzz::shape(&rb_face, &[], unicode_buffer);

    for (glyph_pos, glyph_info) in glyph_buffer
        .glyph_positions()
        .iter()
        .zip(glyph_buffer.glyph_infos())
    {
        let advance = font_units_to_px(glyph_pos.x_advance, style.font_size(), &face);
        let x_offset = font_units_to_px(glyph_pos.x_offset, style.font_size(), &face);
        let y_offset = font_units_to_px(glyph_pos.y_offset, style.font_size(), &face);
        items.push(Item::Glyph(ShapedGlyph {
            glyph: GlyphId(glyph_info.glyph_id.try_into().unwrap()),
            source_index: base_index + glyph_info.cluster,
            span_index,
            pos: Vec2::ZERO,
            offset: vec2(x_offset, y_offset),
            advance,
        }));
    }

    // Allow reusing the buffer
    context.cache.unicode_buffer = glyph_buffer.clear();
}

fn compute_size(fonts: &mut FontStore, items: &[Item], styles: &[ResolvedStyle]) -> Vec2 {
    let mut size = Vec2::ZERO;

    for item in items {
        if let Item::Glyph(glyph) = item {
            size.x = size.x.max(glyph.pos.x + glyph.advance);

            let style = &styles[glyph.span_index as usize];
            let font = fonts.get(style.font());
            let line_height = i32::from(font.ascender())
                + i32::from(font.descender().abs())
                + i32::from(font.line_gap());
            let line_height = font_units_to_px(line_height, style.font_size(), font);
            size.y = size.y.max(glyph.pos.y + line_height);
        }
    }

    size
}

fn font_units_to_px(font_units: i32, font_size: f32, font: &impl AsFaceRef) -> f32 {
    font_units as f32 * (points_to_pixels(font_size) / font.as_face_ref().units_per_em() as f32)
}

#[derive(Copy, Clone, Debug)]
pub enum Item {
    /// A glyph character.
    Glyph(ShapedGlyph),
    /// An explicit line break (`\n`).
    ExplicitLineBreak,
}

/// A character in a galley.
#[derive(Copy, Clone, Debug)]
pub struct ShapedGlyph {
    /// The glyph to draw
    glyph: GlyphId,
    /// The index of the glyph into the source text
    source_index: u32,
    /// The style index in `Galley.span_styles` that indicates
    /// the style of this glyph
    span_index: u32,

    /// Position of the glyph relative to the text blob origin
    pos: Vec2,
    /// Offset from the pen position to draw at
    offset: Vec2,
    /// X distance to advance the pen after drawing (Y advance unsupported for now)
    advance: f32,
}

impl ShapedGlyph {
    pub fn glyph_id(&self) -> GlyphId {
        self.glyph
    }

    pub fn source_index(&self) -> usize {
        self.source_index as usize
    }

    pub fn span_index(&self) -> usize {
        self.span_index as usize
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn offset(&self) -> Vec2 {
        self.offset
    }
}
