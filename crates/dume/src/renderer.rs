use crate::{
    path::{Path, TesselateKind},
    Context, FontId, Rect, TextureId, TextureSetId,
};

use ahash::AHashMap;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec4};
use swash::GlyphId;
use wgpu::util::DeviceExt;

use self::{
    layering::LayeringEngine,
    path::{PathBatch, PathRenderer, PreparedPathBatch},
    sprite::{PreparedSpriteBatch, SpriteBatch, SpriteRenderer},
    text::{PreparedTextBatch, TextBatch, TextRenderer},
};

mod layering;
mod path;
mod sprite;
mod text;

pub use path::Paint;

/// Renderer for a canvas.
///
/// # Batched rendering
/// Rendering is split into _batches_ -
/// each type of primitive that can be rendered
/// belongs to a single batch. A batch can be drawn
/// with a single draw call, using a single shader and vertex buffer.
///
/// The renderer uses a tile-based hit detection algorithm to
/// determine where batches intersect. To ensure proper draw order,
/// a primitive has to be added to a new batch if the old batch of the
/// same type was occluded by a primitive in a different, more recent batch.
///
/// For example:
/// 1. render fullscreen image -> batch 1
/// 2. render another fullscreen image -> batch 1
/// 3. render some text on top -> batch 2 (because the primitive type is different)
/// 4. render another fullscreen image -> batch 3 (because if added to batch 1,
///    the text would be drawn over the image, not under it
///    as draw order requires)
/// 5. render another fullsreen image, but from a different texture set -> batch 4
///    (because the bind groups provided
///    to the shader are different for different texture sets)
pub struct Renderer {
    sprite_renderer: SpriteRenderer,
    text_renderer: TextRenderer,
    path_renderer: PathRenderer,

    batches: Batches,

    layering: LayeringEngine,
}

pub struct PreparedRender {
    batches: Vec<PreparedBatch>,
}

enum PreparedBatch {
    Sprite(PreparedSpriteBatch),
    Text(PreparedTextBatch),
    Path(PreparedPathBatch),
}

impl Renderer {
    pub fn new(device: &wgpu::Device, window_size: Vec2) -> Self {
        let mut layering = LayeringEngine::new();
        layering.set_window_size(window_size);
        Self {
            sprite_renderer: SpriteRenderer::new(device),
            text_renderer: TextRenderer::new(device),
            path_renderer: PathRenderer::new(device),
            batches: Batches::default(),
            layering,
        }
    }

    pub fn draw_sprite(&mut self, cx: &Context, texture: TextureId, pos: Vec2, width: f32) {
        let texture_set = cx.textures().set_for_texture(texture);

        let batch_id = find_batch_with_layering(
            &mut self.batches,
            &mut self.layering,
            BatchKey::Sprite { texture_set },
            Batch::Sprite(self.sprite_renderer.create_batch(texture_set)),
            self.sprite_renderer
                .affected_region(cx, texture_set, texture, pos, width),
        );

        self.sprite_renderer.draw_sprite(
            cx,
            self.batches.get(batch_id).unwrap_sprite(),
            texture,
            pos,
            width,
        );
    }

    pub fn draw_glyph(
        &mut self,
        cx: &Context,
        hidpi_factor: f32,
        glyph: GlyphId,
        pos: Vec2,
        size: f32,
        font: FontId,
        color: Vec4,
    ) {
        let batch_id = find_batch_with_layering(
            &mut self.batches,
            &mut self.layering,
            BatchKey::Text,
            Batch::Text(self.text_renderer.create_batch()),
            self.text_renderer
                .affected_region(cx, hidpi_factor, glyph, size, pos, font),
        );

        self.text_renderer.draw_glyph(
            cx,
            hidpi_factor,
            self.batches.get(batch_id).unwrap_text(),
            glyph,
            size,
            color,
            pos,
            font,
        );
    }

    pub fn draw_path(&mut self, cx: &Context, path: &(Path, TesselateKind), paint: Paint) {
        let mut path_cache = cx.path_cache();
        path_cache.with_tesselated_path(path, |tesselated| {
            let batch_id = find_batch_with_layering(
                &mut self.batches,
                &mut self.layering,
                BatchKey::Path,
                Batch::Path(self.path_renderer.create_batch()),
                self.path_renderer.affected_region(tesselated),
            );

            self.path_renderer.draw_path(
                tesselated,
                self.batches.get(batch_id).unwrap_path(),
                paint,
            );
        });
    }

    pub fn prepare_render(
        &mut self,
        cx: &Context,
        device: &wgpu::Device,
        projection_matrix: Mat4,
    ) -> PreparedRender {
        self.batches.by_key.clear();
        self.layering.reset();

        let locals = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&projection_matrix),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let mut prepared = Vec::new();
        for batch in self.batches.batches.drain(..) {
            let prep = match batch {
                Batch::Sprite(s) => PreparedBatch::Sprite(
                    self.sprite_renderer.prepare_batch(cx, device, s, &locals),
                ),
                Batch::Text(s) => {
                    PreparedBatch::Text(self.text_renderer.prepare_batch(cx, device, s, &locals))
                }
                Batch::Path(s) => PreparedBatch::Path(self.path_renderer.prepare_batch(
                    cx,
                    device,
                    cx.queue(),
                    s,
                    &locals,
                )),
            };
            prepared.push(prep);
        }
        self.batches.by_key.clear();

        PreparedRender { batches: prepared }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        prepared: &PreparedRender,

        target_texture: &wgpu::TextureView,
        target_sample_texture: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: target_sample_texture,
                resolve_target: Some(target_texture),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        for prep in &prepared.batches {
            match prep {
                PreparedBatch::Sprite(s) => self.sprite_renderer.render_layer(&mut render_pass, s),
                PreparedBatch::Text(s) => self.text_renderer.render_layer(&mut render_pass, s),
                PreparedBatch::Path(s) => self.path_renderer.render_layer(&mut render_pass, s),
            }
        }
    }

    pub fn resize(&mut self, new_target_size: Vec2) {
        self.layering.set_window_size(new_target_size);
    }
}

fn find_batch_with_layering(
    batches: &mut Batches,
    layering: &mut LayeringEngine,
    key: BatchKey,
    created_batch: Batch,
    affected_draw_region: Rect,
) -> BatchId {
    let mut created_batch = Some(created_batch);
    let (_, batch_id) = batches.batch_by_key_or_insert(key, || created_batch.take().unwrap());

    let layering_result = layering.layer(affected_draw_region, batch_id);

    match layering_result {
        layering::LayeringResult::UseCurrentBatch => batch_id,
        layering::LayeringResult::CreateNewBatch => {
            batches.insert_batch(key, created_batch.take().unwrap())
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct BatchId(usize);

#[derive(Default)]
struct Batches {
    batches: Vec<Batch>,
    by_key: AHashMap<BatchKey, BatchId>,
}

impl Batches {
    pub fn get(&mut self, id: BatchId) -> &mut Batch {
        &mut self.batches[id.0]
    }

    pub fn batch_by_key_or_insert(
        &mut self,
        key: BatchKey,
        create_batch: impl FnOnce() -> Batch,
    ) -> (&mut Batch, BatchId) {
        match self.by_key.get(&key) {
            Some(&id) => (&mut self.batches[id.0], id),
            None => {
                let batch = create_batch();
                self.batches.push(batch);
                let id = BatchId(self.batches.len() - 1);
                self.by_key.insert(key, id);
                (&mut self.batches[id.0], id)
            }
        }
    }

    pub fn insert_batch(&mut self, key: BatchKey, batch: Batch) -> BatchId {
        self.batches.push(batch);
        let id = BatchId(self.batches.len() - 1);
        self.by_key.insert(key, id);
        id
    }
}

enum Batch {
    Sprite(SpriteBatch),
    Text(TextBatch),
    Path(PathBatch),
}

impl Batch {
    pub fn unwrap_sprite(&mut self) -> &mut SpriteBatch {
        match self {
            Batch::Sprite(s) => s,
            _ => panic!("expected sprite batch"),
        }
    }

    pub fn unwrap_text(&mut self) -> &mut TextBatch {
        match self {
            Batch::Text(s) => s,
            _ => panic!("expected text batch"),
        }
    }

    pub fn unwrap_path(&mut self) -> &mut PathBatch {
        match self {
            Batch::Path(s) => s,
            _ => panic!("expected path batch"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum BatchKey {
    Sprite { texture_set: TextureSetId },
    Text,
    Path,
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
struct Locals {
    projection_matrix: Mat4,
}
