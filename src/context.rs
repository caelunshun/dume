use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard};

use crate::texture::{MissingTexture, TextureId, TextureSet, TextureSetBuilder, Textures};

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
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    textures: RwLock<Textures>,
}

impl Context {
    pub fn create_texture_set_builder(&self) -> TextureSetBuilder {
        TextureSetBuilder::new(self.clone())
    }

    pub fn add_texture_set(&self, set: TextureSet) {
        self.0.textures.write().add_texture_set(set);
    }

    pub fn texture_for_name(&self, name: &str) -> Result<TextureId, MissingTexture> {
        self.0.textures.read().texture_for_name(name)
    }

    pub(crate) fn textures(&self) -> RwLockReadGuard<Textures> {
        self.0.textures.read()
    }

    pub(crate) fn device(&self) -> &Arc<wgpu::Device> {
        &self.0.device
    }

    pub(crate) fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.0.queue
    }
}
