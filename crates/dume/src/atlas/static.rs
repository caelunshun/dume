use std::{collections::BTreeMap, num::NonZeroU32};

use ahash::AHashMap;
use glam::{uvec2, vec2, UVec2, Vec2};
use rectangle_pack::{GroupedRectsToPlace, RectToInsert, TargetBin};

use super::{AtlasEntry, TextureKey};

#[derive(Debug, thiserror::Error)]
#[error("textures cannot fit into texture atlas")]
pub struct NotEnoughSpace;

/// A static texture atlas. Created with a [`StaticTextureAtlasBuilder`].
///
/// Once built, a static texture atlas is immutable. Use `DynamicTextureAtlas`
/// if textures need to be added and removed on the fly.
///
/// The advantage of the static atlas is that it may pack textures more
/// efficiently than the dynamic variant, since it knows all texture sizes upfront.
pub struct StaticTextureAtlas {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    descriptor: wgpu::TextureDescriptor<'static>,
    entries: AHashMap<TextureKey, AtlasEntry>,
}

impl StaticTextureAtlas {
    pub fn builder(format: wgpu::TextureFormat) -> StaticTextureAtlasBuilder {
        StaticTextureAtlasBuilder::new(format)
    }

    pub fn width(&self) -> u32 {
        self.descriptor.size.width
    }

    pub fn height(&self) -> u32 {
        self.descriptor.size.height
    }

    pub fn size(&self) -> UVec2 {
        uvec2(self.width(), self.height())
    }

    pub fn get(&self, key: TextureKey) -> &AtlasEntry {
        &self.entries[&key]
    }

    pub fn texcoords(&self, key: TextureKey) -> [Vec2; 4] {
        let placement = self.get(key);
        let start = placement.pos.as_f32() / self.size().as_f32();
        let size = placement.size.as_f32() / self.size().as_f32();
        [
            start,
            start + vec2(size.x, 0.),
            start + size,
            start + vec2(0., size.y),
        ]
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }
}

struct TextureBuffer {
    data: Vec<u8>,
    size: UVec2,
}

const PADDING: u32 = 2;

/// Builder for a [`StaticTextureAtlas`].
pub struct StaticTextureAtlasBuilder {
    rects: GroupedRectsToPlace<TextureKey>,
    textures: AHashMap<TextureKey, TextureBuffer>,
    format: wgpu::TextureFormat,
}

impl StaticTextureAtlasBuilder {
    fn new(format: wgpu::TextureFormat) -> Self {
        Self {
            rects: GroupedRectsToPlace::new(),
            textures: AHashMap::new(),
            format,
        }
    }

    /// Adds a texture to the atlas.
    ///
    /// `data` must match the `TextureFormat` that the atlas was
    /// initialized with.
    pub fn add_texture(&mut self, data: Vec<u8>, size: UVec2) -> TextureKey {
        let key = TextureKey::new();
        self.textures.insert(key, TextureBuffer { data, size });
        self.rects
            .push_rect(key, None, RectToInsert::new(size.x + PADDING, size.y + PADDING, 1));
        key
    }

    /// Builds the atlas, computing atlas placements and writing texture data to the GPU.
    ///
    /// `min_size` and `max_size` must be powers of two satisfying `max_size >= min_size`.
    ///
    /// Returns an error if the given size constraints are insufficient to fit all textures.
    pub fn build(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        min_size: u32,
        max_size: u32,
    ) -> Result<StaticTextureAtlas, NotEnoughSpace> {
        assert!(min_size.is_power_of_two());
        assert!(max_size.is_power_of_two());
        assert!(min_size <= max_size);

        let mut placements = None;

        // Use the minimum power-of-two size that is able to
        // contains all textures.
        let mut size = min_size;
        while size <= max_size {
            let mut bins = BTreeMap::new();
            bins.insert((), TargetBin::new(size, size, 1));
            match rectangle_pack::pack_rects(
                &self.rects,
                &mut bins,
                &rectangle_pack::volume_heuristic,
                &rectangle_pack::contains_smallest_box,
            ) {
                Ok(rects) => {
                    placements = Some(rects);
                    break;
                }
                Err(rectangle_pack::RectanglePackError::NotEnoughBinSpace) => {}
            }

            size *= 2;
        }

        let placements = match placements {
            Some(p) => p,
            None => return Err(NotEnoughSpace),
        };

        let descriptor = wgpu::TextureDescriptor {
            label: Some("static_texture_atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&descriptor);

        // Write texture data
        let mut entries = AHashMap::new();
        for (key, buffer) in self.textures {
            let placement = placements.packed_locations().get(&key).unwrap().1;

            entries.insert(
                key,
                AtlasEntry {
                    pos: uvec2(placement.x() + PADDING / 2, placement.y() + PADDING / 2),
                    size: buffer.size,
                },
            );

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: placement.x() + PADDING / 2,
                        y: placement.y() + PADDING / 2,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &buffer.data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        NonZeroU32::new(self.format.describe().block_size as u32 * buffer.size.x)
                            .expect("texture width cannot be zero"),
                    ),
                    rows_per_image: Some(
                        NonZeroU32::new(buffer.size.y).expect("texture height cannot be zero"),
                    ),
                },
                wgpu::Extent3d {
                    width: buffer.size.x,
                    height: buffer.size.y,
                    depth_or_array_layers: 1,
                },
            );
        }

        Ok(StaticTextureAtlas {
            texture_view: texture.create_view(&Default::default()),
            texture,
            descriptor,
            entries,
        })
    }
}
