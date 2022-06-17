use std::{mem, num::NonZeroU32};

use ahash::AHashMap;
use fast_image_resize::{FilterType, Image, MulDiv, PixelType, ResizeAlg, Resizer};
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
        for (name, texture) in &set.by_name {
            let id = self.textures_to_sets.insert(texture_set_id);
            if self.by_name.insert(name.clone(), id).is_some() {
                log::warn!("Duplicate textures with name '{}'", name);
            }
            set.by_id.insert(id, texture.clone());
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

#[derive(Debug, Clone)]
pub(crate) struct Texture {
    /// Dimensions of the top mipmap layer.
    size: UVec2,
    /// Texture mipmap levels. mipmap_levels[0] is the full resolution
    /// texture; each subsequent level has half the resolution of the previous
    /// level.
    mipmap_levels: Box<[TextureKey]>,
}

impl Texture {
    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn num_mipmap_levels(&self) -> u32 {
        self.mipmap_levels.len() as u32
    }

    pub fn mipmap_levels(&self) -> &[TextureKey] {
        &self.mipmap_levels
    }

    pub fn mipmap_level(&self, level: u32) -> &TextureKey {
        &self.mipmap_levels[level as usize]
    }

    pub fn mipmap_level_for_target_size(&self, target_width: u32) -> u32 {
        let ratio = target_width as f64 / self.size.x as f64;

        let level = (ratio.log2().abs().floor()) as u32;

        level.min(self.num_mipmap_levels() - 1)
    }
}

/// Stores a group of textures available to a context and its canvases.
///
/// Each texture belongs to a texture set. Each texture set
/// is a single texture atlas. For optimal performance, textures
/// that are rendered together should belong to the same texture set.
pub struct TextureSet {
    atlas: StaticTextureAtlas,
    by_id: SecondaryMap<TextureId, Texture>,
    by_name: AHashMap<String, Texture>,
}

impl TextureSet {
    pub fn atlas(&self) -> &StaticTextureAtlas {
        &self.atlas
    }

    pub(crate) fn get(&self, id: TextureId) -> &Texture {
        &self.by_id[id]
    }
}

/// Builder for a texture set.
pub struct TextureSetBuilder {
    context: Context,
    atlas_builder: StaticTextureAtlasBuilder,
    by_name: AHashMap<String, Texture>,

    // Used for mipmap generation
    resizer: Resizer,
    mul_div: MulDiv,
}

impl TextureSetBuilder {
    pub(crate) fn new(context: Context) -> Self {
        Self {
            context,
            atlas_builder: StaticTextureAtlas::builder(TEXTURE_FORMAT),
            by_name: AHashMap::new(),

            resizer: Resizer::new(ResizeAlg::Convolution(FilterType::Hamming)), // Hamming best for downsampling
            mul_div: MulDiv::default(),
        }
    }

    /// Adds a texture to the texture set from raw RGBA data.
    ///
    /// `data` should be _BGRA_-encoded 8-bit texture data.
    /// Note that most texture data you work with will be RGBA;
    /// you will need to swap the byte order when passing data to this function.
    ///
    /// This function will generate mipmaps for the texture.
    pub fn add_raw_texture(
        &mut self,
        width: u32,
        height: u32,
        data: Vec<u8>,
        name: impl Into<String>,
    ) {
        let size = uvec2(width, height);
        let key = self.atlas_builder.add_texture(data.clone(), size);

        // Generate mipmap layers
        let mut mipmap_levels = vec![key];
        let mut current_level_size = size;
        let mut current_image = Image::from_vec_u8(
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
            data,
            PixelType::U8x4,
        )
        .expect("dimensions do not match image data size");

        while current_level_size.x > 1
            && current_level_size.y > 1
            && (mipmap_levels.len() as u32) < self.context.settings().max_mipmap_levels
        {
            self.mul_div
                .multiply_alpha_inplace(&mut current_image.view_mut())
                .expect("failed to premultiply alpha");

            let next_level_size = current_level_size / 2;
            let mut next_image = Image::new(
                NonZeroU32::new(next_level_size.x).unwrap(),
                NonZeroU32::new(next_level_size.y).unwrap(),
                PixelType::U8x4,
            );
            self.resizer
                .resize(&current_image.view(), &mut next_image.view_mut())
                .unwrap();

            self.mul_div
                .divide_alpha_inplace(&mut next_image.view_mut())
                .expect("failed to unpremultiply alpha");

            mipmap_levels.push(
                self.atlas_builder
                    .add_texture(next_image.buffer().to_vec(), next_level_size),
            );

            current_level_size = next_level_size;
            current_image = next_image;
        }

        let texture = Texture {
            size,
            mipmap_levels: mipmap_levels.into_boxed_slice(),
        };
        self.by_name.insert(name.into(), texture);
    }

    /// Adds a texture to the texture set from an encoded image.
    #[cfg(feature = "image_")]
    pub fn add_texture(
        &mut self,
        data: &[u8],
        name: impl Into<String>,
    ) -> Result<(), image::ImageError> {
        let image = image::load_from_memory(data)?.to_rgba8();
        let width = image.width();
        let height = image.height();
        let mut bytes = image.into_raw();
        rgba_to_bgra(&mut bytes);
        self.add_raw_texture(width, height, bytes, name);
        Ok(())
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
        })
    }
}

fn rgba_to_bgra(rgba: &mut [u8]) {
    rgba.chunks_exact_mut(4).for_each(|chunk| {
        if let [r, _g, b, _a] = chunk {
            mem::swap(r, b);
        }
    })
}
