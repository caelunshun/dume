use std::{convert::TryInto, mem::size_of};

use bytemuck::{Pod, Zeroable};
use glam::{uvec2, vec4, Affine2, UVec2, Vec2, Vec4};
use wgpu::util::DeviceExt;

use crate::{path::TesselatedPath, Context, Rect, SAMPLE_COUNT, TARGET_FORMAT};

use super::Locals;

#[derive(Copy, Clone, Debug)]
pub enum Paint {
    SolidColor(Vec4),
    LinearGradient {
        p_a: Vec2,
        p_b: Vec2,
        c_a: Vec4,
        c_b: Vec4,
    },
    RadialGradient {
        center: Vec2,
        radius: f32,
        c_center: Vec4,
        c_outer: Vec4,
    },
}

impl Paint {
    pub fn encode(&self, color_buffer: &mut Vec<Vec4>) -> UVec2 {
        let index = color_buffer.len() as u32;
        // Type constants must match those defined in path.wgsl
        let typ = match self {
            Paint::SolidColor(color) => {
                color_buffer.push(*color);
                0
            }
            Paint::LinearGradient { p_a, p_b, c_a, c_b } => {
                color_buffer.push(*c_a);
                color_buffer.push(*c_b);
                color_buffer.push(vec4(p_a.x, p_a.y, p_b.x, p_b.y));
                1
            }
            Paint::RadialGradient {
                center,
                radius,
                c_center,
                c_outer,
            } => {
                color_buffer.push(*c_center);
                color_buffer.push(*c_outer);
                color_buffer.push(vec4(center.x, center.y, *radius, 0.));
                2
            }
        };
        uvec2(typ, index)
    }
}

/// Renderer for vector paths - both stroking and filling.
pub struct PathRenderer {
    bg_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl PathRenderer {
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
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

        let module = device.create_shader_module(&wgpu::include_wgsl!("../../shaders/path.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("path_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Sint32x2],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
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

        Self {
            bg_layout,
            pipeline,
        }
    }

    pub fn create_batch(&self) -> PathBatch {
        PathBatch {
            vertices: Vec::new(),
            indices: Vec::new(),
            colors: Vec::new(),
        }
    }

    pub fn affected_region(&self, path: &TesselatedPath) -> Rect {
        // Compute path boundary box
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);

        for vertex in &path.vertices {
            if vertex.x < min.x {
                min.x = vertex.x;
            }
            if vertex.y < min.y {
                min.y = vertex.y;
            }
            if vertex.x > max.x {
                max.x = vertex.x;
            }
            if vertex.y > max.y {
                max.y = vertex.y;
            }
        }

        Rect::new(min, max - min)
    }

    pub fn draw_path(
        &self,
        path: &TesselatedPath,
        transform: Affine2,
        batch: &mut PathBatch,
        paint: Paint,
    ) {
        let paint = paint.encode(&mut batch.colors);

        let base_vertex = batch.vertices.len() as u32;
        for vertex in &path.vertices {
            batch.vertices.push(Vertex {
                position: transform.transform_point2(*vertex),
                local_position: *vertex,
                paint,
            });
        }
        batch
            .indices
            .extend(path.indices.iter().map(|&i| i + base_vertex));
    }

    pub fn prepare_batch(
        &self,
        _cx: &Context,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layer: PathBatch,
        locals: &wgpu::Buffer,
    ) -> PreparedPathBatch {
        let colors = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("colors"),
                size: wgpu::Extent3d {
                    width: layer.colors.len() as u32,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
            },
            bytemuck::cast_slice(&layer.colors),
        );
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
                    resource: wgpu::BindingResource::TextureView(
                        &colors.create_view(&Default::default()),
                    ),
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

        PreparedPathBatch {
            bind_group,
            vertices,
            indices,
            num_indices,
        }
    }

    pub fn render_layer<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        layer: &'a PreparedPathBatch,
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
    position: Vec2,
    local_position: Vec2,
    paint: UVec2,
}

pub struct PathBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    colors: Vec<Vec4>,
}

pub struct PreparedPathBatch {
    bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32,
}
