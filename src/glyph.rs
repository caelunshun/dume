use std::{collections::hash_map::Entry, sync::Arc};

use ahash::AHashMap;
use fontdb::Database;
use fontdue::layout::GlyphRasterConfig;
use guillotiere::{AllocId, Allocation};

use crate::{atlas::TextureAtlas, text::FontId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub c: char,
    pub font: FontId,
    pub size: u64, // fixed point in 1/1000s of a pixel
}

/// A cache of rendered glyphs stored on a GPU texture atlas.
pub struct GlyphCache {
    atlas: TextureAtlas,
    fonts: AHashMap<FontId, fontdue::Font>,
    glyphs: AHashMap<GlyphKey, Allocation>,
}

impl GlyphCache {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let atlas = TextureAtlas::new(device, queue, wgpu::TextureFormat::R8Unorm, "font_atlas");
        Self {
            atlas,
            fonts: AHashMap::new(),
            glyphs: AHashMap::new(),
        }
    }

    pub fn atlas(&self) -> &TextureAtlas {
        &self.atlas
    }

    /// Gets the atlas allocation for the given glyph. Returns `None` if the glyph
    /// is empty and should not be rendered.
    pub fn glyph_allocation(&mut self, key: GlyphKey, fonts: &Database) -> Option<Allocation> {
        match self.glyphs.entry(key) {
            Entry::Occupied(entry) => Some(*entry.get()),
            Entry::Vacant(entry) => {
                let font = match self.fonts.entry(key.font) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => {
                        let (font, _index) = fonts.face_source(key.font).unwrap();
                        let font_data = match &*font {
                            fontdb::Source::Binary(b) => b.as_slice(),
                            fontdb::Source::File(_) => todo!(),
                        };
                        let font = fontdue::Font::from_bytes(
                            font_data,
                            fontdue::FontSettings {
                                enable_offset_bounding_box: false,
                                ..Default::default()
                            },
                        )
                        .expect("malformed font");
                        entry.insert(font)
                    }
                };

                let (metrics, alpha_map) = font.rasterize(key.c, key.size as f32 / 1000.);

                if metrics.width == 0 || metrics.height == 0 {
                    return None;
                }

                Some(*entry.insert(self.atlas.insert(
                    &alpha_map,
                    metrics.width as u32,
                    metrics.height as u32,
                )))
            }
        }
    }
}
