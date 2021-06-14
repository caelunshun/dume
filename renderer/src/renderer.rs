use std::{mem::size_of, sync::Arc};

use bytemuck::{Pod, Zeroable};
use fontdb::Database;
use glam::{vec2, IVec2, Mat4, Vec2, Vec4};
use guillotiere::Allocation;
use palette::Srgba;
use wgpu::util::DeviceExt;

use crate::{
    glyph::{GlyphCache, GlyphKey},
    sprite::{SpriteId, Sprites},
    TARGET_FORMAT,
};

// Must match uber.frag.glsl defines
const PAINT_SOLID_COLOR: i32 = 0;
const PAINT_SPRITE: i32 = 1;
const PAINT_ALPHA_TEXTURE: i32 = 2;

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    pos: Vec2,
    texcoord: Vec2,
    paint: IVec2,
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    ortho: Mat4,
}

pub struct PreparedRender {
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    color_buffer: wgpu::Buffer,

    num_indices: u32,
}

/// Renders sprites, glyphs, and paths using a single draw call.
///
/// Calling `record_*` adds another command, which is
/// buffered in a vertex buffer. Calling `render` causes
/// the sprites to be rendered.
pub struct Renderer {
    sprites: Sprites,
    glyphs: GlyphCache,

    device: Arc<wgpu::Device>,
    #[allow(unused)]
    queue: Arc<wgpu::Queue>,
    sampler: wgpu::Sampler,
    nearest_sampler: wgpu::Sampler,
    pipeline: wgpu::RenderPipeline,
    bg_layout: wgpu::BindGroupLayout,

    /// Buffered for the current layer.
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    colors: Vec<Vec4>,
}

impl Renderer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let (pipeline, bg_layout) = create_pipeline(&device);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });
        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("font_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });
        Self {
            sprites: Sprites::new(Arc::clone(&device), Arc::clone(&queue)),
            glyphs: GlyphCache::new(Arc::clone(&device), Arc::clone(&queue)),

            device,
            queue,
            sampler,
            nearest_sampler,
            pipeline,
            bg_layout,

            vertices: Vec::new(),
            indices: Vec::new(),
            colors: Vec::new(),
        }
    }

    pub fn sprites(&self) -> &Sprites {
        &self.sprites
    }

    pub fn sprites_mut(&mut self) -> &mut Sprites {
        &mut self.sprites
    }

    /// Draws a sprite on the current layer.
    pub fn record_sprite(&mut self, id: SpriteId, pos: Vec2, width: f32) {
        let allocation = self.sprites.sprite_allocation(id);
        let height =
            width * allocation.rectangle.height() as f32 / allocation.rectangle.width() as f32;
        let size = Vec2::new(width, height);

        let texcoords = self.sprites().atlas().texture_coordinates(allocation);

        let paint = glam::ivec2(PAINT_SPRITE, 0);

        self.push_quad(pos, size, texcoords, paint);
    }

    pub fn record_glyph(&mut self, key: GlyphKey, pos: Vec2, color: Srgba<u8>, fonts: &Database) {
        if let Some(allocation) = self.glyphs.glyph_allocation(key, fonts) {
            let texcoords = self.glyphs.atlas().texture_coordinates(allocation);

            let paint = glam::ivec2(PAINT_ALPHA_TEXTURE, self.push_color(color));

            let size = vec2(
                allocation.rectangle.size().width as f32 - 2.0,
                allocation.rectangle.size().height as f32 - 2.0,
            );

            self.push_quad(pos, size, texcoords, paint);
        }
    }

    fn push_color(&mut self, color: Srgba<u8>) -> i32 {
        let linear = color.into_format::<f32, f32>().into_linear();
        self.colors.push(Vec4::new(
            linear.red,
            linear.green,
            linear.blue,
            linear.alpha,
        ));

        self.colors.len() as i32 - 1
    }

    fn push_quad(&mut self, pos: Vec2, size: Vec2, texcoords: [Vec2; 4], paint: IVec2) {
        let vertices = [
            Vertex {
                pos,
                texcoord: texcoords[0],
                paint,
            },
            Vertex {
                pos: pos + Vec2::new(size.x, 0.0),
                texcoord: texcoords[1],
                paint,
            },
            Vertex {
                pos: pos + size,
                texcoord: texcoords[2],
                paint,
            },
            Vertex {
                pos: pos + Vec2::new(0.0, size.y),
                texcoord: texcoords[3],
                paint,
            },
        ];
        let i = self.vertices.len() as u16;
        assert!(i.checked_add(4).is_some(), "too many sprites in one layer");
        let indices = [i, i + 1, i + 2, i + 2, i + 3, i];

        self.vertices.extend_from_slice(&vertices);
        self.indices.extend_from_slice(&indices);
    }

    /// Prepares to render the current layer, and flushes the command buffer.
    pub fn prepare(&mut self, ortho: Mat4) -> PreparedRender {
        let uniforms = Uniforms { ortho };
        let uniform_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsage::UNIFORM,
            });

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsage::INDEX,
            });

        if self.colors.is_empty() {
            self.colors.push(glam::vec4(1.0, 1.0, 1.0, 1.0));
        }

        let color_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("colors"),
                contents: bytemuck::cast_slice(&self.colors),
                usage: wgpu::BufferUsage::STORAGE,
            });

        let num_indices = self.indices.len() as u32;
        self.vertices.clear();
        self.indices.clear();
        self.colors.clear();

        let sprite_tv = self
            .sprites()
            .atlas()
            .texture()
            .create_view(&Default::default());

        let font_tv = self
            .glyphs
            .atlas()
            .texture()
            .create_view(&Default::default());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.nearest_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&sprite_tv),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&font_tv),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &color_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        PreparedRender {
            bind_group,
            vertex_buffer,
            index_buffer,
            color_buffer,
            num_indices,
        }
    }

    pub fn render<'pass>(
        &'pass mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        data: &'pass mut PreparedRender,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
        pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_bind_group(0, &data.bind_group, &[]);
        pass.draw_indexed(0..data.num_indices, 0, 0..1);
    }
}

fn create_pipeline(device: &wgpu::Device) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    // Validation needs to be disabled for non-uniformity in the fragment shader (?)
    let vert_mod = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        flags: wgpu::ShaderFlags::default(),
        ..wgpu::include_spirv!("../shader/uber.vert.spv")
    });
    let frag_mod = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        flags: wgpu::ShaderFlags::default(),
        ..wgpu::include_spirv!("../shader/uber.frag.spv")
    });

    let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    filtering: true,
                    comparison: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    filtering: false,
                    comparison: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bg_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("sprite_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vert_mod,
            entry_point: "main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: size_of::<Vertex>() as _,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Sint32x2],
            }],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            clamp_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &frag_mod,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: TARGET_FORMAT,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    }
                }),
                write_mask: wgpu::ColorWrite::ALL,
            }],
        }),
    });

    (render_pipeline, bg_layout)
}
