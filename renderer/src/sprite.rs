use std::sync::Arc;

use ahash::AHashMap;
use glam::UVec2;
use guillotiere::Allocation;
use image::{
    imageops::{self, FilterType},
    Bgra, ImageBuffer,
};
use slotmap::{SecondaryMap, SlotMap};

use crate::atlas::TextureAtlas;

const MIPMAP_LEVELS: u32 = 4;

slotmap::new_key_type! {
    /// Unique ID of a sprite.

    pub struct SpriteId;
}

#[derive(Debug, Clone)]
pub struct SpriteInfo {
    /// Size in pixels.
    pub size: UVec2,
    // One entry for each mipmap level, where
    // level 0 is the largest. mipmap_sizes[0] == size
    pub mipmap_sizes: Vec<UVec2>,

    /// The atlas allocation storing each mipmap level.
    pub mipmap_allocations: Vec<Allocation>,
}

/// Stores available sprite textures.
pub struct Sprites {
    atlas: TextureAtlas,
    infos: SlotMap<SpriteId, SpriteInfo>,
    by_name: AHashMap<String, SpriteId>,
}

impl Sprites {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            atlas: TextureAtlas::new(
                device,
                queue,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                "sprite_atlas",
            ),
            infos: SlotMap::default(),
            by_name: AHashMap::new(),
        }
    }

    /// Inserts a new sprite from RGBA data. (sRGB, not linear)
    pub fn insert(
        &mut self,
        rgba_data: &mut [u8],
        width: u32,
        height: u32,
        name: String,
    ) -> SpriteId {
        // Make data BGRA.
        rgba_data.chunks_exact_mut(4).for_each(|chunk| {
            if let [r, g, b, a] = *chunk {
                chunk.copy_from_slice(&[b, g, r, a]);
            }
        });
        let bgra_data = rgba_data;

        let mut current_image: ImageBuffer<Bgra<u8>, _> =
            ImageBuffer::from_raw(width, height, bgra_data.to_vec()).unwrap();
        let mut current_size = UVec2::new(width, height);

        // Write each mipmap level.
        let mut info = SpriteInfo {
            size: current_size,
            mipmap_sizes: Vec::new(),
            mipmap_allocations: Vec::new(),
        };

        for _ in 0..MIPMAP_LEVELS {
            let allocation =
                self.atlas
                    .insert(current_image.as_raw(), current_size.x, current_size.y);
            info.mipmap_allocations.push(allocation);
            info.mipmap_sizes.push(current_size);

            // Downscale to next mipmap level.
            current_size = UVec2::new((current_size.x / 2).max(1), (current_size.y / 2).max(1));
            current_image = imageops::resize(
                &current_image,
                current_size.x,
                current_size.y,
                FilterType::Triangle,
            );
        }

        let sprite_id = self.infos.insert(info);
        self.by_name.insert(name, sprite_id);
        sprite_id
    }

    pub fn remove(&mut self, id: SpriteId) {
        let info = self.infos.remove(id);
        if let Some(info) = info {
            let name = self
                .by_name
                .iter()
                .find(|(_, i)| **i == id)
                .unwrap()
                .0
                .clone();
            self.by_name.remove(&name);

            for mipmap_allocation in info.mipmap_allocations {
                self.atlas.remove(mipmap_allocation);
            }
        }
    }

    pub fn sprite_info(&self, id: SpriteId) -> &SpriteInfo {
        &self.infos[id]
    }

    /// Determines the mipmap level to use when a
    /// sprite is rendered at the given width.
    pub fn mipmap_level_for_scale(&self, id: SpriteId, target_width: f32) -> usize {
        let info = self.sprite_info(id);
        let width = info.size.x as f32;

        let width_ratio = width / target_width;
        width_ratio
            .log2()
            .floor()
            .clamp(0.0, MIPMAP_LEVELS as f32 - 1.0) as usize
    }

    pub fn sprite_by_name(&self, name: &str) -> Option<SpriteId> {
        self.by_name.get(name).copied()
    }

    /// Gets the sprite texture atlas.
    pub fn atlas(&self) -> &TextureAtlas {
        &self.atlas
    }

    pub fn sprite_size(&self, id: SpriteId) -> UVec2 {
        self.sprite_info(id).size
    }
}
