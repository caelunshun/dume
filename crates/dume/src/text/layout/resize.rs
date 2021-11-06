use ahash::AHashSet;
use glam::Vec2;
use parking_lot::RwLockReadGuard;

use crate::{font::Fonts, text::layout::GlyphCharacter, Context, TextBlob};

#[derive(Default)]
struct Line {
    next_line_offset: f32,
    num_glyphs: usize,
}

/// Does line wrapping on a list of `ShapedGlyph`.
pub struct Layouter<'a> {
    blob: &'a mut TextBlob,
    fonts: RwLockReadGuard<'a, Fonts>,

    cursor: Vec2,

    current_line: Line,

    next_glyph: usize,

    used_line_breaks: AHashSet<u32>,
    previous_word_break: Option<usize>,
}

impl<'a> Layouter<'a> {
    pub fn new(blob: &'a mut TextBlob, cx: &'a Context) -> Self {
        blob.size = Vec2::ZERO;
        Self {
            blob,
            fonts: cx.fonts(),

            cursor: Vec2::ZERO,

            current_line: Line::default(),

            next_glyph: 0,

            used_line_breaks: AHashSet::new(),
            previous_word_break: None,
        }
    }

    fn next_line(&mut self) {
        self.blob.size.x = self.blob.size.x.max(self.cursor.x);
        self.cursor.x = 0.;
        self.cursor.y += self.current_line.next_line_offset;
        self.blob.size.y += self.current_line.next_line_offset;
        self.current_line.num_glyphs = 0;
    }

    fn process_next_glyph(&mut self) {
        if matches!(
            &self.blob.glyphs[self.next_glyph].c,
            GlyphCharacter::LineBreak
        ) {
            self.next_line();
        }

        let glyph = &mut self.blob.glyphs[self.next_glyph];

        glyph.pos = self.cursor;

        self.cursor.x += glyph.advance;

        // Apply the line offset for this line
        let font = self.fonts.get(glyph.font);
        let metrics = font.metrics(&[]);
        let font_scale = glyph.size / metrics.units_per_em as f32;
        let line_offset = font_scale * (metrics.ascent + metrics.descent + metrics.leading);
        self.current_line.next_line_offset = self.current_line.next_line_offset.max(line_offset);

        if let GlyphCharacter::Glyph(_, _, c) = &glyph.c {
            if *c == ' '
                && !self
                    .used_line_breaks
                    .contains(&(self.next_glyph as u32 + 1))
            {
                self.previous_word_break = Some(self.next_glyph + 1);
            }
        }

        if self.cursor.x > self.blob.max_size.x && self.current_line.num_glyphs > 0 {
            // We need to wrap to the next line.
            //
            // If there is an available word boundary, we move back
            // to the boundary before wrapping.
            if let Some(boundary) = self.previous_word_break.take() {
                if self.used_line_breaks.insert(boundary as u32) {
                    self.next_glyph = boundary;
                }
            }

            self.next_line();
        } else {
            self.next_glyph += 1;
            self.current_line.num_glyphs += 1;
        }
    }

    pub fn run_layout(mut self) {
        while self.next_glyph < self.blob.glyphs.len() {
            self.process_next_glyph();
        }

        self.next_line();
    }
}
