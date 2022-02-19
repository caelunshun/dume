use std::{mem::size_of, num::NonZeroU64};

use bytemuck::{Pod, Zeroable};
use glam::{uvec2, vec2, UVec2, Vec2};
use palette::Srgba;
use wgpu::util::DeviceExt;

use crate::{Context, Rect, TextureSetId, INTERMEDIATE_FORMAT, TARGET_FORMAT};

// Must match definitions in render.wgsl.
const TILE_WORKGROUP_SIZE: u32 = 256;
const SORT_WORKGROUP_SIZE: u32 = 16;
const TILE_SIZE: u32 = 16;

const SHAPE_FILL_RECT: i32 = 0;
const SHAPE_STROKE_RECT: i32 = 1;
const SHAPE_FILL_CIRCLE: i32 = 2;
const SHAPE_STROKE_CIRCLE: i32 = 3;
const SHAPE_FILL_PATH: i32 = 4;
const SHAPE_STROKE_PATH: i32 = 5;

const PAINT_TYPE_SOLID: i32 = 0;
const PAINT_TYPE_LINEAR_GRADIENT: i32 = 1;
const PAINT_TYPE_RADIAL_GRADIENT: i32 = 2;
const PAINT_TYPE_GLYPH: i32 = 3;
const PAINT_TYPE_TEXTURE: i32 = 4;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StrokeCap {
    Round = 0,
    Square = 1,
}

/// Drives the GPU renderer.
///
/// One `Renderer` exists per `Context`.
/// To draw onto a canvas, create a `Batch`.
pub struct Renderer {
    pipelines: Pipelines,
    empty_texture: wgpu::TextureView,
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            pipelines: Pipelines::new(device),
            empty_texture: device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("empty_texture"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                })
                .create_view(&Default::default()),
        }
    }

    pub fn create_batch(&self, physical_size: UVec2, scale_factor: f32) -> Batch {
        Batch {
            scale_factor,
            physical_size,
            logical_size: physical_size.as_f32() / scale_factor,

            nodes: Vec::new(),
            node_bounding_boxes: Vec::new(),
            points: Vec::new(),

            texture_set: None,
        }
    }

    pub fn prepare_render(
        &self,
        mut batch: Batch,
        context: &Context,
        target_texture: &wgpu::TextureView,
    ) -> PreparedRender {
        if batch.points.is_empty() {
            batch.points.push(0);
        }

        let device = context.device();

        let glyphs = context.glyph_cache();
        let glyph_atlas = glyphs.atlas().texture_view();

        let globals = batch.globals();
        let globals = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let nodes = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&batch.nodes),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let node_bounding_boxes = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&batch.node_bounding_boxes),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let tile_nodes = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: batch.tile_buffer_size(),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let tile_counters = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: batch.tile_counters_buffer_size(),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let points = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&batch.points),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let textures = context.textures();
        let texture_atlas = match batch.texture_set {
            Some(id) => textures.texture_set(id).atlas().texture_view(),
            None => &self.empty_texture,
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipelines.render_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &globals,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &nodes,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &node_bounding_boxes,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &tile_nodes,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &tile_counters,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(target_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&self.pipelines.linear_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(glyph_atlas),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &points,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(texture_atlas),
                },
            ],
        });

        PreparedRender {
            bind_group,
            tile_count: batch.tile_count(),
            node_count: batch.nodes.len() as u32,
        }
    }

    pub fn render(&self, prepared: PreparedRender, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

        // Tiles
        pass.set_pipeline(&self.pipelines.tile_pipeline);
        pass.set_bind_group(0, &prepared.bind_group, &[]);
        pass.dispatch(
            (prepared.node_count + TILE_WORKGROUP_SIZE - 1) / TILE_WORKGROUP_SIZE,
            1,
            1,
        );

        // Sort
        pass.set_pipeline(&self.pipelines.sort_pipeline);
        pass.set_bind_group(0, &prepared.bind_group, &[]);
        pass.dispatch(
            (prepared.tile_count.x + SORT_WORKGROUP_SIZE - 1) / SORT_WORKGROUP_SIZE,
            (prepared.tile_count.y + SORT_WORKGROUP_SIZE - 1) / SORT_WORKGROUP_SIZE,
            1,
        );

        // Paint
        pass.set_pipeline(&self.pipelines.paint_pipeline);
        pass.set_bind_group(0, &prepared.bind_group, &[]);
        pass.dispatch(prepared.tile_count.x, prepared.tile_count.y, 1);
    }

    pub fn prepare_blit(
        &self,
        context: &Context,
        source: &wgpu::TextureView,
        target_size: UVec2,
    ) -> PreparedBlit {
        let device = context.device();
        let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&target_size),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipelines.blit_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.pipelines.nearest_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ubuf,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });
        PreparedBlit { bind_group }
    }

    pub fn blit(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        prepared: PreparedBlit,
        target: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        pass.set_pipeline(&self.pipelines.blit_pipeline);
        pass.set_bind_group(0, &prepared.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

struct Pipelines {
    tile_pipeline: wgpu::ComputePipeline,
    sort_pipeline: wgpu::ComputePipeline,
    paint_pipeline: wgpu::ComputePipeline,
    render_bg_layout: wgpu::BindGroupLayout,

    blit_pipeline: wgpu::RenderPipeline,
    blit_bg_layout: wgpu::BindGroupLayout,

    nearest_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
}

impl Pipelines {
    pub fn new(device: &wgpu::Device) -> Self {
        let render_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            NonZeroU64::new(size_of::<Globals>() as u64).unwrap(),
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: INTERMEDIATE_FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::COMPUTE,
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
            bind_group_layouts: &[&render_bg_layout],
            push_constant_ranges: &[],
        });

        let render_module =
            device.create_shader_module(&wgpu::include_wgsl!("../shaders/render.wgsl"));
        let tile_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &render_module,
            entry_point: "tile_kernel",
        });
        let paint_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &render_module,
            entry_point: "paint_kernel",
        });
        let sort_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &render_module,
            entry_point: "sort_kernel",
        });

        let blit_module = device.create_shader_module(&wgpu::include_wgsl!("../shaders/blit.wgsl"));
        let blit_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&blit_bg_layout],
            push_constant_ranges: &[],
        });
        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_module,
                entry_point: "vs_main",
                buffers: &[],
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
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_module,
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

        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest_sampler"),
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
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear_sampler"),
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
            tile_pipeline,
            sort_pipeline,
            paint_pipeline,
            render_bg_layout,
            blit_pipeline,
            blit_bg_layout,
            nearest_sampler,
            linear_sampler,
        }
    }
}

#[derive(Pod, Zeroable, Debug, Copy, Clone)]
#[repr(C)]
struct Globals {
    target_size: Vec2,
    tile_count: UVec2,
    node_count: u32,
    scale_factor: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum PaintType {
    Solid(Srgba<u8>),
    LinearGradient {
        point_a: Vec2,
        point_b: Vec2,
        color_a: Srgba<u8>,
        color_b: Srgba<u8>,
    },
    RadialGradient {
        center: Vec2,
        radius: f32,
        color_center: Srgba<u8>,
        color_outer: Srgba<u8>,
    },
    Glyph {
        offset_in_atlas: UVec2,
        origin: UVec2,
        color: Srgba<u8>,
    },
    Texture {
        offset_in_atlas: UVec2,
        origin: Vec2,
        texture_set: TextureSetId,
        scale: f32,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum Shape {
    Rect {
        rect: Rect,
        border_radius: f32,
        stroke_width: Option<f32>,
    },
    Circle {
        center: Vec2,
        radius: f32,
        stroke_width: Option<f32>,
    },
    Stroke {
        segment: LineSegment,
        width: f32,
        cap: StrokeCap,
        path_id: u32,
    },
    Fill {
        segment: LineSegment,
        path_id: u32,
        fill_bounding_box: Rect,
    },
}

#[derive(Copy, Clone, Debug)]
pub struct LineSegment {
    pub start: Vec2,
    pub end: Vec2,
}

#[derive(Debug)]
pub struct Node {
    pub shape: Shape,
    pub paint_type: PaintType,
}

impl Node {
    fn bounding_box(&self) -> Rect {
        match self.shape {
            Shape::Rect {
                rect, stroke_width, ..
            } => match stroke_width {
                Some(w) => Rect {
                    pos: rect.pos - w,
                    size: rect.size + 2. * w,
                },
                None => rect,
            },
            Shape::Circle {
                center,
                radius,
                stroke_width,
            } => match stroke_width {
                Some(w) => Rect {
                    pos: center - radius - w,
                    size: Vec2::splat(radius * 2. + w * 2.),
                },
                None => Rect {
                    pos: center - radius,
                    size: Vec2::splat(radius * 2.),
                },
            },
            Shape::Stroke { segment, width, .. } => {
                let min = segment.start.min(segment.end);
                let max = segment.start.max(segment.end);
                Rect {
                    pos: min - width,
                    size: (max - min) + 2. * width,
                }
            }
            Shape::Fill { segment, .. } => {
                let min = segment.start.min(segment.end);
                let max = segment.start.max(segment.end);
                Rect {
                    pos: min,
                    size: max - min,
                }
            }
        }
    }
}

#[derive(Pod, Zeroable, Debug, Copy, Clone, Default)]
#[repr(C)]
struct PackedNode {
    shape: i32,
    pos_a: u32,
    pos_b: u32,
    extra: u32,

    paint_type: i32,
    color_a: u32,
    color_b: u32,
    gradient_point_a: u32,
    gradient_point_b: u32,
}

#[derive(Pod, Zeroable, Debug, Copy, Clone)]
#[repr(C)]
struct PackedBoundingBox {
    pos: u32,
    size: u32,
}

/// A batch is a list of draw commands prepared
/// for rendering in one compute pass.
pub struct Batch {
    nodes: Vec<PackedNode>,
    node_bounding_boxes: Vec<PackedBoundingBox>,
    points: Vec<u32>,

    physical_size: UVec2,
    logical_size: Vec2,
    scale_factor: f32,

    texture_set: Option<TextureSetId>,
}

impl Batch {
    pub fn draw_node(&mut self, node: Node) {
        let bbox = node.bounding_box();
        if !self.will_draw(bbox) {
            return;
        }

        let node = self.pack_node(node);
        self.nodes.push(node);
        self.node_bounding_boxes.push(self.pack_bounding_box(bbox));
    }

    /// Clipping step on the CPU.
    fn will_draw(&self, bbox: Rect) -> bool {
        let min = bbox.pos;
        let max = min + bbox.size;
        !(max.x < 0. || max.y < 0. || min.x > self.logical_size.x || min.y > self.logical_size.y)
    }

    pub fn logical_size(&self) -> Vec2 {
        self.logical_size
    }

    pub fn physical_size(&self) -> UVec2 {
        self.physical_size
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    fn tile_count(&self) -> UVec2 {
        (self.physical_size + UVec2::splat(TILE_SIZE - 1)) / UVec2::splat(TILE_SIZE)
    }

    fn tile_buffer_size(&self) -> u64 {
        let num_tiles = self.tile_count().x * self.tile_count().y;
        (num_tiles as u64) * 64 * (size_of::<u32>() as u64)
    }

    fn tile_counters_buffer_size(&self) -> u64 {
        let num_tiles = self.tile_count().x * self.tile_count().y;
        num_tiles as u64 * size_of::<u32>() as u64
    }

    fn globals(&self) -> Globals {
        Globals {
            target_size: self.logical_size,
            tile_count: self.tile_count(),
            node_count: self
                .nodes
                .len()
                .try_into()
                .expect("how did you draw 2^32 nodes?"),
            scale_factor: self.scale_factor,
        }
    }

    fn pack_pos(&self, pos: Vec2) -> u32 {
        let pos = pos.clamp(-self.logical_size * 0.5, self.logical_size * 1.5);
        let x = (((pos.x + self.logical_size.x / 2.) / (2. * self.logical_size.x))
            * (u16::MAX as f32)) as u16;
        let y = (((pos.y + self.logical_size.y / 2.) / (2. * self.logical_size.y))
            * (u16::MAX as f32)) as u16;
        x as u32 | ((y as u32) << 16)
    }

    fn pack_upos(&self, pos: UVec2) -> u32 {
        pos.x | (pos.y << 16)
    }

    fn pack_color(&self, color: Srgba<u8>) -> u32 {
        color.red as u32
            | ((color.green as u32) << 8)
            | ((color.blue as u32) << 16)
            | ((color.alpha as u32) << 24)
    }

    fn pack_node(&mut self, node: Node) -> PackedNode {
        let mut packed = PackedNode::default();

        match node.shape {
            Shape::Rect {
                rect,
                border_radius,
                stroke_width,
            } => {
                packed.shape = if stroke_width.is_some() {
                    SHAPE_STROKE_RECT
                } else {
                    SHAPE_FILL_RECT
                };
                packed.pos_a = self.pack_pos(rect.pos);
                packed.pos_b = self.pack_pos(rect.size);
                packed.extra = self.pack_pos(vec2(border_radius, stroke_width.unwrap_or(0.)));
            }
            Shape::Circle {
                center,
                radius,
                stroke_width,
            } => {
                packed.shape = if stroke_width.is_some() {
                    SHAPE_STROKE_CIRCLE
                } else {
                    SHAPE_FILL_CIRCLE
                };
                packed.pos_a = self.pack_pos(center);
                packed.pos_b = self.pack_pos(vec2(radius, stroke_width.unwrap_or(0.)));
            }
            Shape::Stroke {
                segment,
                width,
                cap,
                path_id,
            } => {
                packed.shape = SHAPE_STROKE_PATH;

                let base_index = self.points.len() as u32;
                self.points.push(self.pack_pos(segment.start));
                self.points.push(self.pack_pos(segment.end));

                packed.pos_a = self.pack_upos(uvec2(base_index, 0));
                packed.pos_b = self.pack_pos(vec2(width, (cap as u32) as f32));
                packed.extra = path_id;
            }
            Shape::Fill {
                segment,
                path_id,
                fill_bounding_box,
            } => {
                packed.shape = SHAPE_FILL_PATH;

                let base_index = self.points.len() as u32;
                self.points
                    .push(self.pack_upos(segment.start.round().as_u32()));
                self.points
                    .push(self.pack_upos(segment.end.round().as_u32()));

                let packed_bbox = self.pack_bounding_box(fill_bounding_box);
                self.points.push(packed_bbox.pos);
                self.points.push(packed_bbox.size);

                packed.pos_a = self.pack_upos(uvec2(base_index, base_index + 2));
                packed.extra = path_id;
            }
        }

        match node.paint_type {
            PaintType::Solid(color) => {
                packed.paint_type = PAINT_TYPE_SOLID;
                packed.color_a = self.pack_color(color);
            }
            PaintType::LinearGradient {
                point_a,
                point_b,
                color_a,
                color_b,
            } => {
                packed.paint_type = PAINT_TYPE_LINEAR_GRADIENT;
                packed.color_a = self.pack_color(color_a);
                packed.color_b = self.pack_color(color_b);
                packed.gradient_point_a = self.pack_pos(point_a);
                packed.gradient_point_b = self.pack_pos(point_b);
            }
            PaintType::RadialGradient {
                center,
                radius,
                color_center,
                color_outer,
            } => {
                packed.paint_type = PAINT_TYPE_RADIAL_GRADIENT;
                packed.color_a = self.pack_color(color_center);
                packed.color_b = self.pack_color(color_outer);
                packed.gradient_point_a = self.pack_pos(center);
                packed.gradient_point_b = self.pack_pos(vec2(radius, 0.));
            }
            PaintType::Glyph {
                offset_in_atlas,
                origin,
                color,
            } => {
                packed.paint_type = PAINT_TYPE_GLYPH;
                packed.color_a = self.pack_color(color);
                packed.gradient_point_a = self.pack_upos(offset_in_atlas);
                packed.gradient_point_b = self.pack_upos(origin);
            }
            PaintType::Texture {
                offset_in_atlas,
                origin,
                texture_set,
                scale,
            } => {
                packed.paint_type = PAINT_TYPE_TEXTURE;
                packed.gradient_point_a = self.pack_upos(offset_in_atlas);
                packed.gradient_point_b = self.pack_pos(origin);
                packed.color_a = self.pack_pos(vec2(scale, 0.));

                match self.texture_set {
                    Some(id) => assert_eq!(
                        id, texture_set,
                        "using multiple texture sets in one batch is unimplemented"
                    ),
                    None => self.texture_set = Some(texture_set),
                }
            }
        }

        packed
    }

    fn pack_bounding_box(&self, bbox: Rect) -> PackedBoundingBox {
        PackedBoundingBox {
            pos: self.pack_pos(bbox.pos),
            size: self.pack_pos(bbox.size),
        }
    }
}

/// A render that is ready to be fed to a `CommandEncoder`.
pub struct PreparedRender {
    bind_group: wgpu::BindGroup,
    tile_count: UVec2,
    node_count: u32,
}

pub struct PreparedBlit {
    bind_group: wgpu::BindGroup,
}
