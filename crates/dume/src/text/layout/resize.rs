use std::ops::Range;

use ahash::AHashSet;
use glam::Vec2;
use parking_lot::RwLockReadGuard;

use crate::{font::Fonts, text::layout::GlyphCharacter, Align, Baseline, Context, TextBlob};

#[derive(Default, Clone)]
struct Line {
    next_line_offset: f32,
    num_glyphs: usize,
    range: Range<usize>,
    width: f32,
}

/// Does line wrapping on a list of `ShapedGlyph`.
pub struct Layouter<'a> {
    blob: &'a mut TextBlob,
    fonts: RwLockReadGuard<'a, Fonts>,

    cursor: Vec2,

    current_line: Line,
    lines: Vec<Line>,

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
            lines: Vec::new(),

            next_glyph: 0,

            used_line_breaks: AHashSet::new(),
            previous_word_break: None,
        }
    }

    fn next_line(&mut self) {
        self.current_line.range.end = self.next_glyph;
        self.lines.push(self.current_line.clone());

        self.blob.size.x = self.blob.size.x.max(self.cursor.x);
        self.cursor.x = 0.;
        self.cursor.y += self.current_line.next_line_offset;
        self.blob.size.y += self.current_line.next_line_offset;
        self.current_line.num_glyphs = 0;
        self.current_line.range.start = self.next_glyph;
        self.current_line.width = 0.;
    }

    fn process_next_glyph(&mut self) {
        if matches!(
            &self.blob.glyphs[self.next_glyph].c,
            GlyphCharacter::LineBreak
        ) {
            self.next_line();
        }

        let glyph = &mut self.blob.glyphs[self.next_glyph];

        // Apply the line offset for this line
        let font = self.fonts.get(glyph.font);
        let metrics = font.metrics(&[]);
        let font_scale = glyph.size / metrics.units_per_em as f32;
        let line_offset = font_scale * (metrics.ascent + metrics.descent + metrics.leading);
        self.current_line.next_line_offset = self.current_line.next_line_offset.max(line_offset);

        glyph.pos = self.cursor;

        // Account for the baseline setting
        let baseline_offset = match self.blob.options.baseline {
            Baseline::Top => metrics.ascent,
            Baseline::Middle => (metrics.ascent + metrics.descent) / 2.,
            Baseline::Alphabetic => 0.,
            Baseline::Bottom => metrics.descent,
        };
        glyph.pos.y += baseline_offset * font_scale;

        self.cursor.x += glyph.advance;
        self.current_line.width += glyph.advance;

        if let GlyphCharacter::Glyph(_, _, c) = &glyph.c {
            if *c == ' '
                && !self
                    .used_line_breaks
                    .contains(&(self.next_glyph as u32 + 1))
            {
                self.previous_word_break = Some(self.next_glyph + 1);
            }
        }

        if self.cursor.x > self.blob.max_size.x
            && self.current_line.num_glyphs > 0
            && self.blob.options.wrap_lines
        {
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
        self.apply_align();
    }

    fn apply_align(&mut self) {
        // Apply horizontal alignment.
        for line in &self.lines {
            let line_width = line.width;
            let relative_pos =
                relative_align_pos(self.blob.options.align_h, line_width, self.blob.max_size.x);

            for glyph in &mut self.blob.glyphs[line.range.clone()] {
                glyph.pos.x += relative_pos;
            }
        }

        // Apply vertical alignment.
        let height = self.blob.size().y;
        let relative_pos =
            relative_align_pos(self.blob.options.align_v, height, self.blob.max_size.y);
        for glyph in &mut self.blob.glyphs {
            glyph.pos.y += relative_pos;
        }
    }
}

fn relative_align_pos(align: Align, length: f32, max_length: f32) -> f32 {
    match align {
        Align::Start => 0.0,
        Align::Center => {
            let center_pos = max_length / 2.0;
            center_pos - (length / 2.0)
        }
        Align::End => max_length - length,
    }
}
