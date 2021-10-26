use std::{sync::Arc, time::Duration};

use glam::{uvec2, UVec2, Vec2};
use parking_lot::{RwLock, RwLockReadGuard};

use crate::{
    font::{Font, Fonts, MalformedFont},
    texture::{MissingTexture, TextureId, TextureSet, TextureSetBuilder, Textures},
    Canvas,
};

/// Builder for a [`Context`].
pub struct ContextBuilder {
    settings: Settings,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl ContextBuilder {
    /// Sets the precision of glyph subpixel positioning along each axis.
    /// Higher values mean that glyphs will be rasterized
    /// more precisely, but more glyphs will be generated, causing
    /// higher CPU and texture atlas usage.
    ///
    /// The default value is 2 along the X axis and 4 along the Y axis.
    pub fn glyph_subpixel_steps(mut self, steps: UVec2) -> Self {
        assert!(steps.x > 0 && steps.y > 0);
        self.settings.glyph_subpixel_steps = steps;
        self
    }

    /// Sets the duration before an unused glyph is evicted from the texture atlas,
    /// freeing space for other glyphs.
    ///
    /// The default is 10 seconds.
    pub fn glyph_expire_duration(mut self, duration: Duration) -> Self {
        self.settings.glyph_expire_duration = duration;
        self
    }

    /// Builds the context.
    pub fn build(self) -> Context {
        Context(Arc::new(Inner {
            settings: self.settings,

            device: self.device,
            queue: self.queue,

            textures: RwLock::new(Textures::default()),
            fonts: RwLock::new(Fonts::default()),
        }))
    }
}

#[derive(Debug)]
pub(crate) struct Settings {
    pub(crate) glyph_subpixel_steps: UVec2,
    pub(crate) glyph_expire_duration: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            glyph_subpixel_steps: uvec2(2, 4),
            glyph_expire_duration: Duration::from_secs(10),
        }
    }
}

/// The thread-safe Dume context. Stores all images,
/// fonts, and GPU state needed for rendering.
///
/// The `Context` can be cloned to create a new handle.
/// It internally uses an `Arc`.
///
/// To draw to a window or layer, call [`create_canvas`].
#[derive(Clone)]
pub struct Context(Arc<Inner>);

struct Inner {
    settings: Settings,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    textures: RwLock<Textures>,
    fonts: RwLock<Fonts>,
}

impl Context {
    pub fn builder(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> ContextBuilder {
        ContextBuilder {
            settings: Settings::default(),
            device,
            queue,
        }
    }

    pub fn create_texture_set_builder(&self) -> TextureSetBuilder {
        TextureSetBuilder::new(self.clone())
    }

    pub fn add_texture_set(&self, set: TextureSet) {
        self.0.textures.write().add_texture_set(set);
    }

    pub fn texture_for_name(&self, name: &str) -> Result<TextureId, MissingTexture> {
        self.textures().texture_for_name(name)
    }

    pub fn add_font(&self, font_data: Vec<u8>) -> Result<(), MalformedFont> {
        self.0.fonts.write().add(Font::from_data(font_data)?);
        Ok(())
    }

    pub fn set_default_font_family(&self, family: impl Into<String>) {
        self.0.fonts.write().set_default_family(family.into());
    }

    pub fn create_canvas(&self, target_size: Vec2) -> Canvas {
        Canvas::new(self.clone(), target_size)
    }

    pub(crate) fn textures(&self) -> RwLockReadGuard<Textures> {
        self.0.textures.read()
    }

    pub(crate) fn fonts(&self) -> RwLockReadGuard<Fonts> {
        self.0.fonts.read()
    }

    pub(crate) fn device(&self) -> &Arc<wgpu::Device> {
        &self.0.device
    }

    pub(crate) fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.0.queue
    }

    pub(crate) fn settings(&self) -> &Settings {
        &self.0.settings
    }
}
