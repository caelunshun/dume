use std::ops::Range;

use fontdb::Database;
use glam::{vec2, Vec2};
use palette::Srgba;
use rustybuzz::{Direction, UnicodeBuffer};
use unicode_bidi::{BidiInfo, Level};

use crate::{
    font::Font,
    rect::Rect,
    sprite::{SpriteId, Sprites},
    Text, TextSection, TextStyle,
};

use super::FontId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Align {
    /// Top or left
    Start,
    /// Middle or center
    Center,
    /// Bottom or right
    End,
}

/// Defines the baseline of a line of text.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Baseline {
    Top,
    Middle,
    Alphabetic,
    Bottom,
}

/// Settings for laying out text.
///
/// TODO: should some parameters be moved to the rich text
/// representation, so that alignments can be mixed within a paragraph?
#[derive(Debug, Clone)]
#[repr(C)]
pub struct TextLayout {
    /// The maximum dimensions of the formatted text.
    ///
    /// Excess text is hidden.
    pub max_dimensions: Vec2,
    /// Whether to overflow onto a new line when the maximum width is reached.
    ///
    /// If false, then excess characters are omitted.
    pub line_breaks: bool,
    /// The baseline to use.
    pub baseline: Baseline,
    /// Horizontal alignment to apply to the text.
    pub align_h: Align,
    /// Vertical alignment to apply to the text.
    pub align_v: Align,
}

/// A glyph that has been shaped and formatted inside a [`Paragraph`].
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct ShapedGlyph {
    /// Position of the glyph relative to the position of the paragraph.
    pub pos: Vec2,
    /// How far to advance the cursor when drawing this glyph.
    pub advance: Vec2,
    /// Offset from the cursor position to draw the glyph at.
    pub offset: Vec2,
    /// The glyph bounding box.
    pub bbox: Rect,
    /// Offset of the glyph bounding box from the glyph position.
    pub bearing: Vec2,
    /// Whether the glyph is visible and should be drawn.
    pub visible: bool,

    /// Character or icon to draw.
    pub c: GlyphCharacter,
    /// Color of the glyph.
    pub color: Srgba<u8>,
    /// Font size for the glyph.
    pub size: f32,
    /// Font to use for the glyph.
    pub font: Option<FontId>,
}

/// Metrics for a line within a paragraph.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct LineMetrics {
    /// The position of the start of the line. (top-left)
    pub start: Vec2,
    /// The position of the end of the line. (top-right)
    pub end: Vec2,
    /// The range of glyph indices on this line.
    pub range: Range<usize>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GlyphCharacter {
    Char(char),
    Icon(SpriteId),
}

/// A paragraph of rich text that has been layed
/// out and is ready for rendering.
#[derive(Debug, Clone)]
pub struct Paragraph {
    text: Text,
    layout: TextLayout,
    glyphs: Vec<ShapedGlyph>,
    lines: Vec<LineMetrics>,
}

impl Paragraph {
    pub(crate) fn new(text: Text, layout: TextLayout, fonts: &Database, sprites: &Sprites) -> Self {
        let mut glyphs = shape(&text, fonts, sprites);
        let lines = lay_out(&mut glyphs, &layout, fonts);

        Self {
            text,
            layout,
            glyphs,
            lines,
        }
    }

    /// Updates the paragraph's maximum width and height, re-calculating
    /// layout (but not shaping) if needed.
    pub(crate) fn update_max_dimensions(&mut self, fonts: &Database, new_max_dimensions: Vec2) {
        if new_max_dimensions == self.layout.max_dimensions {
            return;
        }

        self.layout.max_dimensions = new_max_dimensions;
        self.lines = lay_out(&mut self.glyphs, &self.layout, fonts);
    }

    /// Gets the glyphs in the paragraph.
    pub fn glyphs(&self) -> &[ShapedGlyph] {
        &self.glyphs
    }

    /// Gets the lines in the paragraph.
    pub fn lines(&self) -> &[LineMetrics] {
        &self.lines
    }

    /// Gets the width of the paragraph.
    pub fn width(&self) -> f32 {
        self.glyphs
            .iter()
            .map(|glyph| (glyph.pos.x + glyph.advance.x) as u32)
            .max()
            .unwrap_or_default() as f32
            - self
                .glyphs
                .iter()
                .map(|glyph| glyph.pos.x as u32)
                .min()
                .unwrap_or_default() as f32
    }

    /// Gets the height of the paragraph.
    pub fn height(&self) -> f32 {
        self.glyphs
            .iter()
            .map(|glyph| (glyph.pos.y + glyph.bbox.size.y) as u32)
            .max()
            .unwrap_or_default() as f32
    }
}

/// Shapes a text without calculating line breaks.
///
//// The `pos` fields of the returned glyphs are set to 0.
fn shape(text: &Text, fonts: &Database, sprites: &Sprites) -> Vec<ShapedGlyph> {
    let mut glyphs = Vec::with_capacity(128);
    for section in text.sections() {
        match section {
            TextSection::Text { text, style } => {
                // Shape the text.
                let bidi_info = BidiInfo::new(text, Some(Level::ltr()));

                let font_id = style
                    .font
                    .with_fontdb_family(|query| fonts.query(query))
                    .unwrap_or_else(|| panic!("no fonts matched the query {:#?}", style.font));
                let (source, face_index) = fonts.face_source(font_id).unwrap();

                let font_data = match &*source {
                    fontdb::Source::Binary(b) => b.as_slice(),
                    fontdb::Source::File(p) => todo!(),
                };
                let font = Font::new(font_data).expect("malformed font");

                for paragraph in &bidi_info.paragraphs {
                    let (levels, runs) = bidi_info.visual_runs(paragraph, paragraph.range.clone());

                    for run in runs {
                        let level = levels[run.start];
                        let subtext = &text[run.clone()];

                        glyphs.extend(shape_word(subtext, style, level, &font, font_id));
                    }
                }
            }
            TextSection::Icon { name, size } => {
                let sprite = sprites
                    .sprite_by_name(name)
                    .or_else(|| sprites.sprite_by_name(&format!("icon/{}", name)))
                    .unwrap_or_else(|| panic!("no sprite with name '{}' or 'icon/{}'", name, name));
                let info = sprites.sprite_info(sprite);
                let width = size * info.size.x as f32 / info.size.y as f32;

                glyphs.push(ShapedGlyph {
                    pos: Vec2::ZERO,
                    advance: vec2(width, 0.0),
                    offset: Vec2::ZERO,
                    bbox: Rect {
                        pos: Vec2::ZERO,
                        size: vec2(width, *size),
                    },
                    bearing: Vec2::ZERO,
                    visible: false,
                    c: GlyphCharacter::Icon(sprite),
                    color: Default::default(), // ignored for sprites
                    size: *size,
                    font: None,
                });
            }
        }
    }

    glyphs
}

fn shape_word(
    word: &str,
    style: &TextStyle,
    level: Level,
    font: &Font,
    font_id: FontId,
) -> Vec<ShapedGlyph> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(word);
    buffer.set_direction(if level.is_rtl() {
        Direction::RightToLeft
    } else {
        Direction::LeftToRight
    });
    let glyph_buffer = rustybuzz::shape(&font.hb, &[], buffer);
    let positions = glyph_buffer.glyph_positions();
    let infos = glyph_buffer.glyph_infos();

    let mut glyphs = Vec::with_capacity(positions.len());

    let scale = style.size / font.ttf.units_per_em().expect("no units per EM") as f32;

    for (position, (_info, c)) in positions.iter().zip(infos.iter().zip(word.chars())) {
        let bbox = font
            .ttf
            .glyph_index(c)
            .map(|id| {
                let bbox = font.ttf.glyph_bounding_box(id).unwrap_or(ttf_parser::Rect {
                    x_min: 0,
                    x_max: 0,
                    y_min: 0,
                    y_max: 0,
                });
                Rect {
                    pos: vec2(bbox.x_min as f32, bbox.y_min as f32) * Vec2::splat(scale),
                    size: vec2(bbox.width() as f32, bbox.height() as f32) * Vec2::splat(scale),
                }
            })
            .unwrap_or(Rect {
                pos: Vec2::ZERO,
                size: Vec2::ZERO,
            });

        let glyph = ShapedGlyph {
            pos: Vec2::ZERO,
            advance: vec2(position.x_advance as f32, position.y_advance as f32)
                * Vec2::splat(scale),
            offset: vec2(position.x_offset as f32, position.y_offset as f32) * Vec2::splat(scale),
            bbox,
            bearing: vec2(bbox.pos.x, bbox.size.y + bbox.pos.y),
            visible: false,
            c: GlyphCharacter::Char(c),
            color: style.color,
            size: style.size,
            font: Some(font_id),
        };
        glyphs.push(glyph);
    }

    glyphs
}

/// Lays out some already-shaped text, applying position data
/// and line breaks.
fn lay_out(glyphs: &mut [ShapedGlyph], layout: &TextLayout, fonts: &Database) -> Vec<LineMetrics> {
    let mut cursor = Vec2::ZERO; // origin relative to paragraph

    // Reset positions in case we don't update them all.
    glyphs.iter_mut().for_each(|g| {
        g.pos = Vec2::ZERO;
        g.visible = false;
    });

    let mut i = 0;
    let mut previous_word_boundary = None;
    let mut lines = vec![LineMetrics::default()];

    let mut max_y = 0.0f32;
    while i < glyphs.len() {
        let was_line_break = i > 0 && glyphs[i - 1].c == GlyphCharacter::Char('\n');
        let glyph = &mut glyphs[i];

        let (source, face_index) = fonts
            .face_source(glyph.font.unwrap_or_else(|| fonts.faces()[0].id))
            .unwrap();

        let font_data = match &*source {
            fontdb::Source::Binary(b) => b.as_slice(),
            fontdb::Source::File(p) => todo!(),
        };
        let font = Font::new(font_data).expect("malformed font");

        let scale = glyph.size / font.ttf.units_per_em().unwrap() as f32;
        let ascender = font.ttf.ascender() as f32 * scale;
        let descender = font.ttf.descender() as f32 * scale;
        let line_gap = font.ttf.line_gap() as f32 * scale;
        let line_space = ascender - descender + line_gap;

        let mut glyph_offset = glyph.offset + vec2(glyph.bearing.x, -glyph.bearing.y);

        let baseline_offset = match layout.baseline {
            Baseline::Top => ascender,
            Baseline::Middle => (ascender + descender) / 2.0,
            Baseline::Alphabetic => 0.0,
            Baseline::Bottom => descender,
        };
        glyph_offset.y += baseline_offset;

        if (cursor + glyph_offset).y > layout.max_dimensions.y {
            // Out of vertical space; abort.
            break;
        }

        if (cursor + glyph_offset).x > layout.max_dimensions.x || was_line_break {
            if layout.line_breaks {
                // Out of space on this line; proceed to the next one.
                cursor.y += line_space;
                cursor.x = 0.0;
                lines.push(LineMetrics {
                    start: cursor,
                    range: i..i + 1,
                    ..Default::default()
                });
                if !was_line_break {
                    // Go to the previous word and do the line break there.
                    // (We can't break in the middle of a word.)
                    i = match previous_word_boundary {
                        Some(i) => i,
                        None => break,
                    };
                    continue;
                }
            } else {
                break;
            }
        }

        if matches!(
            glyph.c,
            GlyphCharacter::Char(' ') | GlyphCharacter::Char('\n')
        ) {
            previous_word_boundary = Some(i + 1);
        }

        glyph.pos = cursor + glyph_offset;

        if glyph.c != GlyphCharacter::Char('\n') {
            glyph.visible = true;
        }
        cursor += glyph.advance;

        let current_line = lines.last_mut().unwrap();
        current_line.end = cursor;
        current_line.range.end = current_line.range.end.max(i + 1);

        max_y = max_y.max(cursor.y + line_space);

        i += 1;
    }

    // Apply horizontal alignment.
    for line in &lines {
        let line_width = line.end.x - line.start.x;
        let relative_pos = relative_align_pos(layout.align_h, line_width, layout.max_dimensions.x);

        for glyph in &mut glyphs[line.range.clone()] {
            glyph.pos.x += relative_pos;
        }
    }

    // Apply vertical alignment.
    let height = max_y;
    let relative_pos = relative_align_pos(layout.align_v, height, layout.max_dimensions.y);
    for glyph in glyphs {
        glyph.pos.y += relative_pos;
    }

    lines
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
