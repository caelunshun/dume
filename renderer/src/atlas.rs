use std::{iter, num::NonZeroU32, sync::Arc};

use glam::Vec2;
use guillotiere::{Allocation, AtlasAllocator, Size};

const STARTING_DIM: u32 = 1024;

/// A texture atlas.
///
/// A padding of two pixels is inserted between stiched textures
/// to avoid bleeding.
pub struct TextureAtlas {
    descriptor: wgpu::TextureDescriptor<'static>,
    texture: wgpu::Texture,

    allocator: AtlasAllocator,

    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl TextureAtlas {
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
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC,
        };
        let texture = device.create_texture(&descriptor);

        Self {
            descriptor,
            texture,

            allocator: AtlasAllocator::new(Size::new(STARTING_DIM as i32, STARTING_DIM as i32)),

            device,
            queue,
        }
    }

    /// Inserts a new texture, returning its ID.
    ///
    /// The atlas is grown if necessary.
    pub fn insert(&mut self, texture: &[u8], width: u32, height: u32) -> Allocation {
        assert_ne!(width, 0, "width cannot be zero");
        assert_ne!(height, 0, "height cannot be zero");
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

        allocation
    }

    /// Deallocates a texture, allowing its space to be reused.
    pub fn remove(&mut self, allocation: Allocation) {
        self.allocator.deallocate(allocation.id);
    }

    /// Gets the texture coordinates into the atlas
    /// of the given allocation.
    ///
    /// Order: top-left, top-right, bottom-right, bottom-left
    pub fn texture_coordinates(&self, allocation: Allocation) -> [Vec2; 4] {
        let min = Vec2::new(
            (allocation.rectangle.min.x as f32 + 1.) / self.descriptor.size.width as f32,
            (allocation.rectangle.min.y as f32 + 1.) / self.descriptor.size.height as f32,
        );
        let max = Vec2::new(
            (allocation.rectangle.max.x as f32 - 1.) / self.descriptor.size.width as f32,
            (allocation.rectangle.max.y as f32 - 1.) / self.descriptor.size.height as f32,
        );
        [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)]
    }

    /// Gets the atlas texture.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn write_texture(&mut self, texture: &[u8], width: u32, height: u32, allocation: Allocation) {
        println!(
            "Writing texture - {}x{} at ({}, {})",
            width, height, allocation.rectangle.min.x, allocation.rectangle.min.y
        );
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: allocation.rectangle.min.x as u32 + 1,
                    y: allocation.rectangle.min.y as u32 + 1,
                    z: 0,
                },
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

        println!("Atlas growing to {}x{}", new_width, new_height);

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
            },
            wgpu::ImageCopyTexture {
                texture: &new_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
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
