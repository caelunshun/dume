//! Text layout implementation.
//!
//! For an overview of the text layout hierarchy,
//! see https://raphlinus.github.io/text/2020/10/26/text-layout.html.

use std::ops::Range;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use swash::{
    text::{cluster::Boundary, Properties, Script},
    GlyphId,
};
use unicode_bidi::{BidiInfo, Level, ParagraphInfo};

use crate::{Context, Text, TextSection, TextStyle, TextureId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub enum Align {
    /// Top or left
    Start,
    /// Middle or center
    Center,
    /// Bottom or right
    End,
}

impl Default for Align {
    fn default() -> Self {
        Align::Start
    }
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

impl Default for Baseline {
    fn default() -> Self {
        Baseline::Alphabetic
    }
}

/// Settings for laying out text.
///
/// TODO: should some parameters be moved to the rich text
/// representation, so that alignments can be mixed within a blob?
#[derive(Debug, Clone)]
pub struct TextOptions {
    /// Whether to overflow onto a new line when the maximum width is reached.
    ///
    /// If false, then excess characters are omitted.
    ///
    /// Line breaks from special characters ('\n') are still respected if this is `false`.
    pub wrap_lines: bool,
    /// The baseline to use.
    pub baseline: Baseline,
    /// Horizontal alignment to apply to the text.
    pub align_h: Align,
    /// Vertical alignment to apply to the text.
    pub align_v: Align,
}

struct CharInfo {
    properties: Properties,
    boundary: Boundary,
}

/// A blob of text that has been laid out and shaped into glyphs.
///
/// Created with [`Context::create_text_blob`](crate::Context::create_text_blob).
///
/// You need to call [`Context::resize_text_blob`] to set the maximum text dimensions
/// and compute glyph layout.
/// If you don't call this method at least once, the text will render as empty.
pub struct TextBlob {
    options: TextOptions,

    pub runs: Vec<BlobRun>,

    /// BiDi info, indexed by byte index
    bidi_levels: Vec<Level>,
    paragraphs: Vec<ParagraphInfo>,

    // Unicode info, indexed by byte index
    char_info: Vec<CharInfo>,

    max_size: Vec2,
    glyphs: Vec<ShapedGlyph>,
}

impl TextBlob {
    pub fn new(cx: &Context, text: Text, options: TextOptions) -> Self {
        let unstyled_text = text.to_unstyled_string();
        let BidiInfo {
            levels, paragraphs, ..
        } = BidiInfo::new(&unstyled_text, None);

        let mut char_info = Vec::with_capacity(unstyled_text.len());
        for ((properties, boundary), c) in
            swash::text::analyze(unstyled_text.chars()).zip(unstyled_text.chars())
        {
            for _ in 0..c.len_utf8() {
                char_info.push(CharInfo {
                    properties,
                    boundary,
                });
            }
        }

        let mut blob = Self {
            options,

            runs: Vec::new(),
            bidi_levels: levels,
            paragraphs,
            char_info,

            max_size: Vec2::ZERO,
            glyphs: Vec::new(),
        };
        blob.compute_runs(cx, text, &unstyled_text);
        blob
    }

    fn compute_runs(&mut self, cx: &Context, text: Text, unstyled_text: &str) {
        // Merge BiDi, style, and script runs.
        let mut byte_index = 0;
        for section in text.sections {
            match section {
                TextSection::Text { text, style } => {
                    self.build_runs(&text, &style, byte_index);
                    byte_index += text.len();
                }
                TextSection::Icon { name, size } => {
                    let texture = cx
                        .texture_for_name(&name)
                        .expect("missing texture for embedded icon in text");
                    self.runs.push(BlobRun::Icon { texture, size });
                }
            }
        }
    }

    fn build_runs(&mut self, text: &str, style: &TextStyle, byte_index: usize) {
        if text.is_empty() {
            return;
        }

        let level_runs = level_runs(&self.bidi_levels[byte_index..(byte_index + text.len())]);
        for level_run in level_runs {
            let start = level_run.start + byte_index;
            let end = level_run.end + byte_index;
            let script_runs = script_runs(&self.char_info[start..end]);

            for (script, script_run) in script_runs {
                let script_start = start + script_run.start;
                let script_end = start + script_run.end;
                self.runs.push(BlobRun::Text {
                    text: (&text[(script_start - byte_index)..(script_end - byte_index)]).into(),
                    style: style.clone(),
                    script,
                    bidi_level: self.bidi_levels[start],
                });
            }
        }
    }

    /// Lays out the text, performing text shaping, line wrapping,
    ///
    pub fn resize(&mut self, cx: &Context, max_size: Vec2) {
        // No need to recompute if the max size hasn't changed by much.
        if !self.should_resize_for(max_size) {
            return;
        }

        self.glyphs.clear();
    }

    fn should_resize_for(&self, max_size: Vec2) -> bool {
        (max_size.x - self.max_size.x).abs() > 0.1 || (max_size.y - self.max_size.y).abs() > 0.1
    }
}

fn level_runs(levels: &[Level]) -> Vec<Range<usize>> {
    let mut result = Vec::new();
    let mut prev_level = levels[0];
    let mut prev_level_start = 0;
    for (i, &level) in levels.iter().enumerate() {
        if level != prev_level {
            result.push(prev_level_start..i);
            prev_level = level;
            prev_level_start = i;
        }
        if i == levels.len() - 1 {
            result.push(prev_level_start..(i + 1));
        }
    }
    result
}

fn script_runs(infos: &[CharInfo]) -> Vec<(Script, Range<usize>)> {
    let mut result = Vec::new();
    let mut prev_script = infos[0].properties.script();
    let mut prev_script_start = 0;
    let mut prev_non_common_script = infos[0].properties.script();
    for (i, info) in infos.iter().enumerate() {
        let script = info.properties.script();

        if script != prev_script && script != Script::Common && prev_script != Script::Common {
            result.push((prev_non_common_script, prev_script_start..i));
            prev_script = script;
            prev_script_start = i;
        }

        if script != Script::Common {
            prev_non_common_script = script;
        }

        if i == infos.len() - 1 {
            let range = prev_script_start..(i + 1);
            result.push((prev_non_common_script, range));
        }
    }
    result
}

/// A glyph in a text blob, ready for rendering or layout.
#[derive(Debug)]
struct ShapedGlyph {
    /// Position of the glyph relative to the text blob origin
    pos: Vec2,
    /// Offset from the pen position to draw at
    offset: Vec2,
    /// X distance to advance the pen after drawing (Y advance unsupported for now)
    advance: f32,

    /// The character to draw
    c: GlyphCharacter,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum GlyphCharacter {
    Glyph(GlyphId),
    Icon(TextureId),
}

/// A run within a [`Blob`] that has the same
/// BiDi level, script, and style.
#[derive(Debug)]
pub enum BlobRun {
    Text {
        text: SmartString<LazyCompact>,
        style: TextStyle,
        bidi_level: Level,
        script: Script,
    },
    Icon {
        texture: TextureId,
        size: f32,
    },
    LineBreak,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_runs() {
        assert_eq!(
            level_runs(&[
                Level::ltr(),
                Level::ltr(),
                Level::rtl(),
                Level::ltr(),
                Level::ltr(),
                Level::rtl(),
            ]),
            vec![0..2, 2..3, 3..5, 5..6]
        );
    }

    #[test]
    fn test_script_runs() {
        let info: Vec<_> = swash::text::analyze("dر".chars())
            .map(|(properties, boundary)| CharInfo {
                properties,
                boundary,
            })
            .collect();

        assert_eq!(
            script_runs(&info),
            vec![(Script::Latin, 0..1), (Script::Arabic, 1..2),]
        );
    }
}
