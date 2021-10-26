use ahash::AHashMap;
use glam::{uvec2, UVec2};
use slotmap::{SecondaryMap, SlotMap};

use crate::{
    atlas::{r#static::NotEnoughSpace, StaticTextureAtlas, StaticTextureAtlasBuilder, TextureKey},
    context::Context,
};

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[derive(Debug, thiserror::Error)]
#[error("missing texture with name '{0}'")]
pub struct MissingTexture(String);

slotmap::new_key_type! {
    /// Unique ID of a texture set.
    pub struct TextureSetId;
}

slotmap::new_key_type! {
    /// ID of a texture.
    pub struct TextureId;
}

#[derive(Default)]
pub(crate) struct Textures {
    sets: SlotMap<TextureSetId, TextureSet>,
    textures_to_sets: SlotMap<TextureId, TextureSetId>,

    by_name: AHashMap<String, TextureId>,
}

impl Textures {
    pub fn add_texture_set(&mut self, set: TextureSet) -> TextureSetId {
        let texture_set_id = self.sets.insert(set);

        let set = &mut self.sets[texture_set_id];
        for (name, key) in &set.by_name {
            let id = self.textures_to_sets.insert(texture_set_id);
            if self.by_name.insert(name.clone(), id).is_some() {
                log::warn!("Duplicate textures with name '{}'", name);
            }
            set.by_id.insert(id, *key);
            set.sizes.insert(id, set.atlas.get(*key).size);
        }

        texture_set_id
    }

    /// Gets a texture by its name
    pub fn texture_for_name(&self, name: &str) -> Result<TextureId, MissingTexture> {
        self.by_name
            .get(name)
            .copied()
            .ok_or_else(|| MissingTexture(name.to_owned()))
    }

    /// Gets the texture set for the given texture
    pub fn set_for_texture(&self, texture: TextureId) -> TextureSetId {
        self.textures_to_sets[texture]
    }

    /// Gets the given texture set
    pub fn texture_set(&self, set: TextureSetId) -> &TextureSet {
        &self.sets[set]
    }
}

/// Stores a group of textures available to a context and its canvases.
///
/// Each texture belongs to a texture set. Each texture set
/// is a single texture atlas. For optimal performance, textures
/// that are rendered together should belong to the same texture set.
pub struct TextureSet {
    atlas: StaticTextureAtlas,
    by_name: AHashMap<String, TextureKey>,
    by_id: SecondaryMap<TextureId, TextureKey>,
    sizes: SecondaryMap<TextureId, UVec2>,
}

impl TextureSet {
    pub fn atlas(&self) -> &StaticTextureAtlas {
        &self.atlas
    }

    pub fn key_by_id(&self, id: TextureId) -> TextureKey {
        self.by_id[id]
    }

    pub fn texture_size(&self, id: TextureId) -> UVec2 {
        self.sizes[id]
    }
}

impl Textures {}

/// Builder for a texture set.
pub struct TextureSetBuilder {
    context: Context,
    atlas_builder: StaticTextureAtlasBuilder,
    by_name: AHashMap<String, TextureKey>,
}

impl TextureSetBuilder {
    pub(crate) fn new(context: Context) -> Self {
        Self {
            context,
            atlas_builder: StaticTextureAtlas::builder(TEXTURE_FORMAT),
            by_name: AHashMap::new(),
        }
    }

    /// Adds a texture to the texture set.
    ///
    /// `data` should be _BGRA_-encoded 8-bit texture data.
    /// Note that most texture data you work with will be RGBA;
    /// you will need to swap the byte order when passing data to this function.
    pub fn add_texture(&mut self, width: u32, height: u32, data: Vec<u8>, name: impl Into<String>) {
        let key = self.atlas_builder.add_texture(data, uvec2(width, height));
        self.by_name.insert(name.into(), key);
    }

    /// Builds the texture set into a texture atlas, copying
    /// all textures to the GPU. Returns the created [`TextureSet`]
    /// which should be added to the `Context`.
    ///
    /// # Panics
    /// Panics if `min_atlas_size > max_atlas_size` or if either atlas
    /// size bound is not a power of two.
    pub fn build(
        self,
        min_atlas_size: u32,
        max_atlas_size: u32,
    ) -> Result<TextureSet, NotEnoughSpace> {
        let atlas = self.atlas_builder.build(
            self.context.device(),
            self.context.queue(),
            min_atlas_size,
            max_atlas_size,
        )?;
        Ok(TextureSet {
            atlas,
            by_name: self.by_name,
            by_id: SecondaryMap::new(), // initialized by Textures::add_texture_set
            sizes: SecondaryMap::new(),
        })
    }
}
