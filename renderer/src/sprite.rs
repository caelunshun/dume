use std::sync::Arc;

use ahash::AHashMap;
use glam::UVec2;
use guillotiere::Allocation;
use slotmap::{SecondaryMap, SlotMap};

use crate::atlas::TextureAtlas;

slotmap::new_key_type! {
    /// Unique ID of a sprite.

    pub struct SpriteId;
}

/// Stores available sprite textures.
pub struct Sprites {
    atlas: TextureAtlas,
    allocations: SlotMap<SpriteId, Allocation>,
    by_name: AHashMap<String, SpriteId>,
    sizes: SecondaryMap<SpriteId, UVec2>,
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
            allocations: SlotMap::default(),
            by_name: AHashMap::new(),
            sizes: SecondaryMap::default(),
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

        let allocation = self.atlas.insert(bgra_data, width, height);
        let sprite_id = self.allocations.insert(allocation);

        self.by_name.insert(name, sprite_id);
        self.sizes.insert(sprite_id, glam::uvec2(width, height));

        sprite_id
    }

    pub fn remove(&mut self, id: SpriteId) {
        if let Some(allocation) = self.allocations.remove(id) {
            self.atlas.remove(allocation);

            let name = self
                .by_name
                .iter()
                .find(|(_, i)| **i == id)
                .unwrap()
                .0
                .clone();
            self.by_name.remove(&name);
        }
    }

    pub fn sprite_allocation(&self, id: SpriteId) -> Allocation {
        self.allocations[id]
    }

    pub fn sprite_by_name(&self, name: &str) -> Option<SpriteId> {
        self.by_name.get(name).copied()
    }

    /// Gets the sprite texture atlas.
    pub fn atlas(&self) -> &TextureAtlas {
        &self.atlas
    }

    pub fn sprite_size(&self, id: SpriteId) -> UVec2 {
        self.sizes[id]
    }
}
