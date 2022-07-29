use std::sync::atomic::{AtomicUsize, Ordering};

use glam::UVec2;

pub mod dynamic;
pub mod r#static;

pub use dynamic::DynamicTextureAtlas;
pub use r#static::{StaticTextureAtlas, StaticTextureAtlasBuilder};

/// Position of a texture within a texture atlas.
///
/// All lengths are in physical pixels.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AtlasEntry {
    /// Offset of the start of the texture from the atlas origin
    pub pos: UVec2,
    /// Size of the texture
    pub size: UVec2,
}

/// A key that uniquely identifies a texture packed in a texture atlas.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]

pub struct TextureKey(usize);

impl TextureKey {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
