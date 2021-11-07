use std::{convert::TryInto, mem::size_of};

use bytemuck::{Pod, Zeroable};
use glam::{vec2, Affine2, Vec2, Vec4};
use swash::GlyphId;
use wgpu::util::DeviceExt;

use crate::{glyph::Glyph, Context, FontId, Rect, SAMPLE_COUNT, TARGET_FORMAT};

use super::Locals;

/// Renderer for text glyphs.
///
/// All glyphs are stored in the same texture atlas in the `GlyphCache`.
pub struct TextRenderer {
    sampler: wgpu::Sampler,
    bg_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("text_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: false,
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

        let module = device.create_shader_module(&wgpu::include_wgsl!("../../shaders/text.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2, 2 => Float32x2],
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

    pub fn create_batch(&self) -> TextBatch {
        TextBatch {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn affected_region(
        &self,
        cx: &Context,
        hidpi_factor: f32,
        glyph: GlyphId,
        size: f32,
        pos: Vec2,
        font: FontId,
    ) -> Rect {
        let mut glyphs = cx.glyph_cache();

        let glyph = glyphs.glyph_or_rasterize(cx, font, glyph, size * hidpi_factor, pos);
        let (_key, placement) = match glyph {
            Glyph::Empty => return Rect::default(),
            Glyph::InAtlas(k, p) => (k, p),
        };

        let pos = pos.floor() + vec2(placement.left as f32, -placement.top as f32) / hidpi_factor;

        let width = placement.width as f32 / hidpi_factor;
        let height = placement.height as f32 / hidpi_factor;

        Rect::new(pos, vec2(width, height))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_glyph(
        &self,
        cx: &Context,
        transform: Affine2,
        hidpi_factor: f32,
        batch: &mut TextBatch,
        glyph: GlyphId,
        size: f32,
        color: Vec4,
        pos: Vec2,
        font: FontId,
    ) {
        let mut glyphs = cx.glyph_cache();

        // Determine scale in transformation
        let scale = hidpi_factor * transform.transform_vector2(Vec2::ONE).x;

        let glyph = glyphs.glyph_or_rasterize(
            cx,
            font,
            glyph,
            size * scale,
            transform.transform_point2(pos),
        );
        let (key, placement) = match glyph {
            Glyph::Empty => return, // nothing to render
            Glyph::InAtlas(k, p) => (k, p),
        };

        let texcoords = glyphs.atlas().texcoords(key);

        let pos = transform.transform_point2(pos) + vec2(placement.left as f32, -placement.top as f32) / hidpi_factor;

        let width = placement.width as f32 / hidpi_factor;
        let height = placement.height as f32 / hidpi_factor;

        let i = batch.vertices.len() as u32;
        batch.vertices.extend_from_slice(&[
            Vertex {
                color,
                pos,
                tex_coords: texcoords[0],
            },
            Vertex {
                color,
                pos: pos + vec2(width, 0.),
                tex_coords: texcoords[1],
            },
            Vertex {
                color,
                pos: pos + vec2(width, height),
                tex_coords: texcoords[2],
            },
            Vertex {
                color,
                pos: pos + vec2(0., height),
                tex_coords: texcoords[3],
            },
        ]);
        batch
            .indices
            .extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 3, i]);
    }

    pub fn prepare_batch(
        &self,
        cx: &Context,
        device: &wgpu::Device,
        layer: TextBatch,
        locals: &wgpu::Buffer,
    ) -> PreparedTextBatch {
        let glyphs = cx.glyph_cache();

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
                    resource: wgpu::BindingResource::TextureView(glyphs.atlas().texture_view()),
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

        PreparedTextBatch {
            bind_group,
            vertices,
            indices,
            num_indices,
        }
    }

    pub fn render_layer<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        layer: &'a PreparedTextBatch,
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
    color: Vec4,
    tex_coords: Vec2,
    pos: Vec2,
}

pub struct TextBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

pub struct PreparedTextBatch {
    bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32,
}
