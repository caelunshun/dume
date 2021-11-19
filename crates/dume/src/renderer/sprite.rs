use std::{convert::TryInto, mem::size_of};

use bytemuck::{Pod, Zeroable};
use glam::{vec2, Affine2, Vec2};
use wgpu::util::DeviceExt;

use crate::{Context, Rect, TextureId, TextureSetId, SAMPLE_COUNT, TARGET_FORMAT};

use super::Locals;

/// Renderer for images sampled from a texture atlas.
pub struct SpriteRenderer {
    sampler: wgpu::Sampler,
    bg_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl SpriteRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.,
            lod_max_clamp: 100.,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some((size_of::<Locals>() as u64).try_into().unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
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

        let module = device.create_shader_module(&wgpu::include_wgsl!("../../shaders/sprite.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
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
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
        });

        Self {
            sampler,
            bg_layout,
            pipeline,
        }
    }

    pub fn create_batch(&self, texture_set: TextureSetId) -> SpriteBatch {
        SpriteBatch {
            vertices: Vec::new(),
            indices: Vec::new(),
            texture_set,
        }
    }

    fn texture_height_and_coords(
        &self,
        cx: &Context,
        transform: Affine2,
        set: TextureSetId,
        texture: TextureId,
        width: f32,
    ) -> (f32, [Vec2; 4]) {
        let scale = transform.transform_vector2(Vec2::ONE).x;

        let textures = cx.textures();
        let set = textures.texture_set(set);
        let texture = set.get(texture);
        let mipmap_level = texture.mipmap_level_for_target_size((scale * width.floor()) as u32);

        let texcoords = set.atlas().texcoords(*texture.mipmap_level(mipmap_level));
        let size = texture.size();
        let aspect_ratio = size.y as f32 / size.x as f32;
        let height = width * aspect_ratio;
        (height, texcoords)
    }

    pub fn affected_region(
        &self,
        cx: &Context,
        transform: Affine2,
        set: TextureSetId,
        texture: TextureId,
        pos: Vec2,
        width: f32,
    ) -> Rect {
        let height = self.texture_height_and_coords(cx, transform, set, texture, width).0;
        Rect::new(pos, vec2(width, height))
    }

    pub fn draw_sprite(
        &self,
        cx: &Context,
        transform: Affine2,
        layer: &mut SpriteBatch,
        texture: TextureId,
        position: Vec2,
        width: f32,
    ) {
        let (height, texcoords) =
            self.texture_height_and_coords(cx, transform, layer.texture_set, texture, width);

        let i = layer.vertices.len() as u32;
        let mut vertices = [
            Vertex {
                position,
                tex_coords: texcoords[0],
            },
            Vertex {
                position: position + vec2(width, 0.),
                tex_coords: texcoords[1],
            },
            Vertex {
                position: position + vec2(width, height),
                tex_coords: texcoords[2],
            },
            Vertex {
                position: position + vec2(0., height),
                tex_coords: texcoords[3],
            },
        ];
        for vert in &mut vertices {
            vert.position = transform.transform_point2(vert.position);
        }

        layer.vertices.extend_from_slice(&vertices);
        layer
            .indices
            .extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 3, i]);
    }

    pub fn prepare_batch(
        &self,
        cx: &Context,
        device: &wgpu::Device,
        layer: SpriteBatch,
        locals: &wgpu::Buffer,
    ) -> PreparedSpriteBatch {
        let textures = cx.textures();
        let texture = textures
            .texture_set(layer.texture_set)
            .atlas()
            .texture_view();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: locals,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(texture),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let num_indices = layer.indices.len() as u32;
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&layer.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&layer.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        PreparedSpriteBatch {
            bind_group,
            vertices,
            indices,
            num_indices,
        }
    }

    pub fn render_layer<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        layer: &'a PreparedSpriteBatch,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &layer.bind_group, &[]);
        pass.set_vertex_buffer(0, layer.vertices.slice(..));
        pass.set_index_buffer(layer.indices.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..layer.num_indices, 0, 0..1);
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec2,
    tex_coords: Vec2,
}

pub struct SpriteBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    texture_set: TextureSetId,
}

pub struct PreparedSpriteBatch {
    bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32,
}
