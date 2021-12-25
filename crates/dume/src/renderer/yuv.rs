use std::{convert::TryInto, mem::size_of, sync::Arc};

use bytemuck::{Pod, Zeroable};
use glam::{vec2, Affine2, Vec2};
use wgpu::util::DeviceExt;

use crate::{Context, YuvTexture, SAMPLE_COUNT, TARGET_FORMAT};

use super::Locals;

/// Renderer for `YuvTexture`s.
pub struct YuvRenderer {
    bg_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}

impl YuvRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
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
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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

        let module = device.create_shader_module(&wgpu::include_wgsl!("../../shaders/yuv.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("yuv_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
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
            multiview: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("yuv_sampler"),
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

        Self {
            bg_layout,
            pipeline,
            sampler,
        }
    }

    pub fn create_batch(&self, texture: &YuvTexture) -> YuvBatch {
        YuvBatch {
            vertices: Vec::new(),
            indices: Vec::new(),
            y_texture: texture.y_texture_view.clone(),
            u_texture: texture.u_texture_view.clone(),
            v_texture: texture.v_texture_view.clone(),
            aspect_ratio: texture.size.y as f32 / texture.size.x as f32,
        }
    }

    pub fn draw_texture(
        &self,
        transform: Affine2,
        batch: &mut YuvBatch,
        position: Vec2,
        width: f32,
        alpha: f32,
    ) {
        let height = width * batch.aspect_ratio;

        let i = batch.vertices.len() as u32;
        let mut vertices = [
            Vertex {
                position,
                tex_coords: vec2(0., 0.),
                alpha,
            },
            Vertex {
                position: position + vec2(width, 0.),
                tex_coords: vec2(1., 0.),
                alpha,
            },
            Vertex {
                position: position + vec2(width, height),
                tex_coords: vec2(1., 1.),
                alpha,
            },
            Vertex {
                position: position + vec2(0., height),
                tex_coords: vec2(0., 1.),
                alpha,
            },
        ];
        for vert in &mut vertices {
            vert.position = transform.transform_point2(vert.position);
        }

        batch.vertices.extend_from_slice(&vertices);
        batch
            .indices
            .extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 3, i]);
    }

    pub fn prepare_batch(
        &self,
        _cx: &Context,
        device: &wgpu::Device,
        batch: YuvBatch,
        locals: &wgpu::Buffer,
    ) -> PreparedYuvBatch {
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
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&batch.y_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&batch.u_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&batch.v_texture),
                },
            ],
        });

        let num_indices = batch.indices.len() as u32;
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&batch.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&batch.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        PreparedYuvBatch {
            bind_group,
            vertices,
            indices,
            num_indices,
        }
    }

    pub fn render_layer<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        layer: &'a PreparedYuvBatch,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &layer.bind_group, &[]);
        pass.set_vertex_buffer(0, layer.vertices.slice(..));
        pass.set_index_buffer(layer.indices.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..layer.num_indices, 0, 0..1);
    }
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    tex_coords: Vec2,
    position: Vec2,
    alpha: f32,
}

pub struct YuvBatch {
    y_texture: Arc<wgpu::TextureView>,
    u_texture: Arc<wgpu::TextureView>,
    v_texture: Arc<wgpu::TextureView>,
    aspect_ratio: f32,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

pub struct PreparedYuvBatch {
    bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32,
}
