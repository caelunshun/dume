use std::{iter, num::NonZeroU32, sync::Arc};

use ahash::AHashMap;
use glam::{uvec2, Vec2};
use guillotiere::{AllocId, Allocation, AtlasAllocator, Size};

use super::{AtlasEntry, TextureKey};

const STARTING_DIM: u32 = 1024;

/// A dynamic texture atlas that supports adding and removing
/// textures on demand. Space from deallocated can be reused.
///
/// A padding of two pixels is inserted between stiched textures
/// to avoid bleeding.
pub struct DynamicTextureAtlas {
    descriptor: wgpu::TextureDescriptor<'static>,
    texture: wgpu::Texture,

    allocator: AtlasAllocator,
    entries: AHashMap<TextureKey, Allocation>,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl DynamicTextureAtlas {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
        label: &'static str,
    ) -> Self {
        let descriptor = wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: STARTING_DIM,
                height: STARTING_DIM,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        };
        let texture = device.create_texture(&descriptor);

        Self {
            descriptor,
            texture,

            allocator: AtlasAllocator::new(Size::new(STARTING_DIM as i32, STARTING_DIM as i32)),
            entries: AHashMap::new(),

            device,
            queue,
        }
    }

    /// Inserts a new texture, returning its ID.
    ///
    /// The atlas is grown if necessary.
    pub fn insert(&mut self, texture: &[u8], width: u32, height: u32) -> TextureKey {
        assert_ne!(width, 0, "width cannot be zero");
        assert_ne!(height, 0, "height cannot be zero");
        let key = TextureKey::new();
        let size = Size::new(width as i32 + 2, height as i32 + 2);
        let allocation = match self.allocator.allocate(size) {
            Some(a) => a,
            None => {
                self.grow(
                    width + self.descriptor.size.width + 2,
                    height + self.descriptor.size.height + 2,
                );
                self.allocator.allocate(size).expect("did not grow")
            }
        };

        self.write_texture(texture, width, height, allocation);

        self.entries.insert(key, allocation);
        key
    }

    /// Deallocates a texture, allowing its space to be reused.
    pub fn remove(&mut self, key: TextureKey) {
        if let Some(alloc) = self.entries.get(&key) {
            self.allocator.deallocate(alloc.id);
        }
    }

    /// Gets a texture's placement in the texture atlas.
    pub fn get(&self, key: TextureKey) -> AtlasEntry {
        let allocation = self.entries[&key];
        let pos = uvec2(
            (allocation.rectangle.min.x + 1) as u32,
            (allocation.rectangle.min.y + 1) as u32,
        );
        let size = uvec2(
            (allocation.rectangle.max.x - allocation.rectangle.min.x - 2) as u32,
            (allocation.rectangle.max.y - allocation.rectangle.min.y - 2) as u32,
        );
        AtlasEntry { pos, size }
    }

    /// Gets the atlas texture.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn write_texture(&mut self, texture: &[u8], width: u32, height: u32, allocation: Allocation) {
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: allocation.rectangle.min.x as u32 + 1,
                    y: allocation.rectangle.min.y as u32 + 1,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            texture,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    NonZeroU32::new(self.descriptor.format.describe().block_size as u32 * width)
                        .expect("width is zero"),
                ),
                rows_per_image: Some(NonZeroU32::new(height).expect("height is zero")),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    fn grow(&mut self, min_width: u32, min_height: u32) {
        let new_width = min_width.next_power_of_two();
        let new_height = min_height.next_power_of_two();

        log::info!("Atlas growing to {}x{}", new_width, new_height);

        self.allocator
            .grow(Size::new(new_width as i32, new_height as i32));

        let wgpu::Extent3d {
            width: old_width,
            height: old_height,
            ..
        } = self.descriptor.size;

        self.descriptor.size.width = new_width;
        self.descriptor.size.height = new_height;

        let new_texture = self.device.create_texture(&self.descriptor);

        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &new_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: old_width,
                height: old_height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(iter::once(encoder.finish()));

        self.texture = new_texture;
    }
}
