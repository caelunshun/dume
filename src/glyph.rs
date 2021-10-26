use std::{sync::Arc, time::Duration};

use glam::{UVec2, Vec2};
use lru::LruCache;
use swash::{
    scale::{Render, ScaleContext, Source},
    zeno::{Format, Placement, Vector},
    GlyphId,
};

use crate::{
    atlas::{DynamicTextureAtlas, TextureKey},
    Context, FontId,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct GlyphKey {
    font: FontId,
    size: u32,              // 1/10s of a pixel
    subpixel_offset: UVec2, // in terms of glyph_subpixel_steps
    glyph_id: GlyphId,
}

#[derive(Debug, Copy, Clone)]
pub enum Glyph {
    Empty, // for unknown glyphs or glyphs with size 0
    InAtlas(TextureKey, Placement),
}

/// A cache of rasterized glyphs stored in a texture atlas.
///
/// Each glyph is uniquely identified by a [`GlyphKey`], which includes
/// the font, size, and subpixel offset of the glyph.
pub struct GlyphCache {
    atlas: DynamicTextureAtlas,
    cache: LruCache<GlyphKey, Glyph>,

    scale_context: ScaleContext,

    glyph_subpixel_steps: UVec2,
    glyph_expire_duration: Duration,
}

impl GlyphCache {
    pub fn new(cx: &Context) -> Self {
        Self {
            atlas: DynamicTextureAtlas::new(
                Arc::clone(cx.device()),
                Arc::clone(cx.queue()),
                wgpu::TextureFormat::R8Unorm,
                "glyph_atlas",
            ),
            cache: LruCache::unbounded(), // glyphs are expired manually

            scale_context: ScaleContext::new(),

            glyph_subpixel_steps: cx.settings().glyph_subpixel_steps,
            glyph_expire_duration: cx.settings().glyph_expire_duration,
        }
    }

    pub fn atlas(&self) -> &DynamicTextureAtlas {
        &self.atlas
    }

    pub fn glyph_or_rasterize(
        &mut self,
        cx: &Context,
        font: FontId,
        glyph_id: GlyphId,
        size: f32,
        position: Vec2,
    ) -> Glyph {
        let subpixel_offset = (position.fract() * self.glyph_subpixel_steps.as_f32()).as_u32();
        let key = GlyphKey {
            font,
            size: (size * 10.) as u32,
            subpixel_offset,
            glyph_id,
        };

        match self.cache.get(&key) {
            Some(g) => *g,
            None => {
                // Rasterize the glyph and write it to the atlas.
                // NB: color bitmaps can't be supported yet because the atlas is alpha-only.
                let mut render = Render::new(&[Source::Outline]);
                render
                    .offset(Vector::new(position.x.fract(), position.y.fract()))
                    .format(Format::Alpha);

                let fonts = cx.fonts();
                let font = fonts.get(font);
                let mut scaler = self
                    .scale_context
                    .builder(font)
                    .hint(true)
                    .size(size)
                    .build();

                let image = render.render(&mut scaler, glyph_id);

                let glyph = match image {
                    Some(image) => {
                        if image.placement.width == 0 || image.placement.height == 0 {
                            Glyph::Empty
                        } else {
                            let key = self.atlas.insert(
                                &image.data,
                                image.placement.width,
                                image.placement.height,
                            );
                            Glyph::InAtlas(key, image.placement)
                        }
                    }
                    None => Glyph::Empty,
                };

                self.cache.put(key, glyph);
                self.cache.get(&key).copied().unwrap()
            }
        }
    }
}
