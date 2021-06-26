//! FFI bindings to dume-renderer.

#![allow(clippy::missing_safety_doc)]

use std::{
    ffi::c_void,
    os::raw::{c_char, c_ulong},
    sync::Arc,
    u64,
};

use glam::Vec2;
use palette::Srgba;
use pollster::block_on;
use raw_window_handle::{unix::XlibHandle, HasRawWindowHandle, RawWindowHandle};
use simple_logger::SimpleLogger;
use slotmap::{Key, KeyData};

use crate::{
    font::{Query, Style, Weight},
    markup, Canvas, Paragraph, Rect, SpriteData, SpriteDescriptor, SpriteId, Text, TextLayout,
    TextStyle, SAMPLE_COUNT, TARGET_FORMAT,
};

pub struct DumeCtx {
    canvas: Canvas,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    sample_texture: wgpu::TextureView,
}

#[repr(C)]
pub struct RawWindow {
    window: c_ulong,
    display: *mut c_void,
}

unsafe impl HasRawWindowHandle for RawWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Xlib(XlibHandle {
            window: self.window,
            display: self.display,
            ..XlibHandle::empty()
        })
    }
}

#[no_mangle]
pub extern "C" fn dume_init(width: u32, height: u32, window: RawWindow) -> *mut DumeCtx {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()
        .unwrap();
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

    let surface = unsafe { instance.create_surface(&window) };

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .expect("failed to find a suitable adapter");

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .expect("failed to get device");
    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: TARGET_FORMAT,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
    let sample_texture = create_sample_texture(&device, &swap_chain_desc);

    let canvas = Canvas::new(Arc::clone(&device), Arc::clone(&queue));

    let ctx = DumeCtx {
        canvas,
        device,
        queue,
        surface,
        swap_chain,
        swap_chain_desc,
        sample_texture,
    };
    Box::leak(Box::new(ctx)) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn dume_resize(ctx: *mut DumeCtx, new_width: u32, new_height: u32) {
    let ctx = unpointer(ctx);

    ctx.swap_chain_desc.width = new_width;
    ctx.swap_chain_desc.height = new_height;
    ctx.swap_chain = ctx
        .device
        .create_swap_chain(&ctx.surface, &ctx.swap_chain_desc);
    ctx.sample_texture = create_sample_texture(&ctx.device, &ctx.swap_chain_desc);
}

#[no_mangle]
pub unsafe extern "C" fn dume_load_font(ctx: *mut DumeCtx, font_data: *const u8, font_len: usize) {
    let data = std::slice::from_raw_parts(font_data, font_len);
    unpointer(ctx).canvas.load_font(data.to_vec());
}

#[no_mangle]
pub unsafe extern "C" fn dume_create_sprite_from_encoded(
    ctx: *mut DumeCtx,
    name: *const u8,
    name_len: usize,
    data: *const u8,
    data_len: usize,
) -> u64 {
    let name = std::str::from_utf8(std::slice::from_raw_parts(name, name_len))
        .expect("invalid UTF-8 in sprite name");
    let data = std::slice::from_raw_parts(data, data_len);

    canvas(ctx)
        .create_sprite(SpriteDescriptor {
            name,
            data: SpriteData::Encoded(data),
        })
        .data()
        .as_ffi()
}

#[no_mangle]
pub unsafe extern "C" fn dume_create_sprite_from_rgba(
    ctx: *mut DumeCtx,
    name: *const u8,
    name_len: usize,
    data: *mut u8,
    data_len: usize,
    width: u32,
    height: u32,
) -> u64 {
    let name = std::str::from_utf8(std::slice::from_raw_parts(name, name_len))
        .expect("invalid UTF-8 in sprite name");
    let data = std::slice::from_raw_parts_mut(data, data_len);

    canvas(ctx)
        .create_sprite(SpriteDescriptor {
            name,
            data: SpriteData::Rgba {
                data,
                width,
                height,
            },
        })
        .data()
        .as_ffi()
}

#[no_mangle]
pub unsafe extern "C" fn dume_get_sprite_by_name(
    ctx: *mut DumeCtx,
    name: *const u8,
    name_len: usize,
) -> u64 {
    canvas(ctx)
        .sprite_by_name(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            name, name_len,
        )))
        .map(|k| k.data().as_ffi())
        .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn dume_get_sprite_size(ctx: *mut DumeCtx, sprite: u64) -> Vec2 {
    let size = canvas(ctx)
        .renderer
        .sprites()
        .sprite_size(SpriteId::from(KeyData::from_ffi(sprite)));
    Vec2::new(size.x as f32, size.y as f32)
}

#[no_mangle]
pub unsafe extern "C" fn dume_get_width(ctx: *mut DumeCtx) -> u32 {
    unpointer(ctx).swap_chain_desc.width
}

#[no_mangle]
pub unsafe extern "C" fn dume_get_height(ctx: *mut DumeCtx) -> u32 {
    unpointer(ctx).swap_chain_desc.height
}

#[repr(C)]
pub struct Variable {
    pub value: *const u8,
    pub len: usize,
}

#[repr(C)]
pub struct CTextStyle {
    pub family_name: *const c_char,
    pub family_name_len: usize,
    pub weight: Weight,
    pub style: Style,
    pub size: f32,
    pub color: *const u8,
}

impl CTextStyle {
    pub unsafe fn to_text_style(&self) -> TextStyle {
        TextStyle {
            color: Srgba::new(
                *self.color,
                *self.color.add(1),
                *self.color.add(2),
                *self.color.add(3),
            ),
            size: self.size,
            font: Query {
                family: std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    self.family_name.cast(),
                    self.family_name_len,
                ))
                .to_owned(),
                style: self.style,
                weight: self.weight,
            },
        }
    }
}

// TEXT
#[no_mangle]
pub unsafe extern "C" fn dume_parse_markup(
    markup: *const u8,
    markup_len: usize,
    default_style: CTextStyle,
    userdata: *mut c_void,
    resolve_variable: extern "C" fn(*mut c_void, *const u8, usize) -> Variable,
) -> *mut Text {
    let markup = std::str::from_utf8_unchecked(std::slice::from_raw_parts(markup, markup_len));
    let text = markup::parse(markup, default_style.to_text_style(), |var| {
        let v = resolve_variable(userdata, var.as_ptr(), var.len());
        if v.value.is_null() {
            panic!("unknown variable '{}'", var);
        }
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(v.value, v.len)).to_owned()
    })
    .expect("failed to parse text");
    Box::leak(Box::new(text)) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn dume_text_free(text: *mut Text) {
    drop(Box::from_raw(text));
}

/// NB: consumes the text.
#[no_mangle]
pub unsafe extern "C" fn dume_create_paragraph(
    ctx: *mut DumeCtx,
    text: *mut Text,
    layout: TextLayout,
) -> *mut Paragraph {
    let text = Box::from_raw(text);
    let paragraph = canvas(ctx).create_paragraph(*text, layout);
    Box::leak(Box::new(paragraph)) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn dume_paragraph_free(paragraph: *mut Paragraph) {
    drop(Box::from_raw(paragraph));
}

#[no_mangle]
pub unsafe extern "C" fn dume_paragraph_resize(
    ctx: *mut DumeCtx,
    paragraph: *mut Paragraph,
    new_max_dimensions: Vec2,
) {
    canvas(ctx).resize_paragraph(unpointer(paragraph), new_max_dimensions);
}

// PAINTING
#[no_mangle]
pub unsafe extern "C" fn dume_draw_sprite(ctx: *mut DumeCtx, pos: Vec2, width: f32, sprite: u64) {
    canvas(ctx).draw_sprite(SpriteId::from(KeyData::from_ffi(sprite)), pos, width);
}

#[no_mangle]
pub unsafe extern "C" fn dume_draw_paragraph(
    ctx: *mut DumeCtx,
    pos: Vec2,
    paragraph: *const Paragraph,
) {
    canvas(ctx).draw_paragraph(pos, &*paragraph);
}

#[no_mangle]
pub unsafe extern "C" fn dume_paragraph_width(p: *const Paragraph) -> f32 {
    (&*p).width()
}

#[no_mangle]
pub unsafe extern "C" fn dume_paragraph_height(p: *const Paragraph) -> f32 {
    (&*p).height()
}

#[no_mangle]
pub unsafe extern "C" fn dume_stroke_width(ctx: *mut DumeCtx, width: f32) {
    canvas(ctx).stroke_width(width);
}

#[no_mangle]
pub unsafe extern "C" fn dume_scissor_rect(ctx: *mut DumeCtx, pos: Vec2, size: Vec2) {
    canvas(ctx).scissor_rect(Rect { pos, size });
}

#[no_mangle]
pub unsafe extern "C" fn dume_clear_scissor(ctx: *mut DumeCtx) {
    canvas(ctx).clear_scissor();
}

#[no_mangle]
pub unsafe extern "C" fn dume_begin_path(ctx: *mut DumeCtx) {
    canvas(ctx).begin_path();
}

#[no_mangle]
pub unsafe extern "C" fn dume_move_to(ctx: *mut DumeCtx, pos: Vec2) {
    canvas(ctx).move_to(pos);
}

#[no_mangle]
pub unsafe extern "C" fn dume_line_to(ctx: *mut DumeCtx, pos: Vec2) {
    canvas(ctx).line_to(pos);
}

#[no_mangle]
pub unsafe extern "C" fn dume_quad_to(ctx: *mut DumeCtx, control: Vec2, pos: Vec2) {
    canvas(ctx).quad_to(control, pos);
}

#[no_mangle]
pub unsafe extern "C" fn dume_cubic_to(
    ctx: *mut DumeCtx,
    control1: Vec2,
    control2: Vec2,
    pos: Vec2,
) {
    canvas(ctx).cubic_to(control1, control2, pos);
}

#[no_mangle]
pub unsafe extern "C" fn dume_arc(
    ctx: *mut DumeCtx,
    center: Vec2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
) {
    canvas(ctx).arc(center, radius, start_angle, end_angle);
}

#[no_mangle]
pub unsafe extern "C" fn dume_solid_color(ctx: *mut DumeCtx, color: &[u8; 4]) {
    canvas(ctx).solid_color(srgba(*color));
}

#[no_mangle]
pub unsafe extern "C" fn dume_linear_gradient(
    ctx: *mut DumeCtx,
    point_a: Vec2,
    point_b: Vec2,
    color_a: &[u8; 4],
    color_b: &[u8; 4],
) {
    canvas(ctx).linear_gradient(point_a, point_b, srgba(*color_a), srgba(*color_b));
}

#[no_mangle]
pub unsafe extern "C" fn dume_stroke(ctx: *mut DumeCtx) {
    canvas(ctx).stroke();
}

#[no_mangle]
pub unsafe extern "C" fn dume_fill(ctx: *mut DumeCtx) {
    canvas(ctx).fill();
}

#[no_mangle]
pub unsafe extern "C" fn dume_translate(ctx: *mut DumeCtx, vector: Vec2) {
    canvas(ctx).translate(vector);
}

#[no_mangle]
pub unsafe extern "C" fn dume_scale(ctx: *mut DumeCtx, scale: f32) {
    canvas(ctx).scale(scale);
}

#[no_mangle]
pub unsafe extern "C" fn dume_reset_transform(ctx: *mut DumeCtx) {
    canvas(ctx).reset_transform();
}

fn srgba(a: [u8; 4]) -> Srgba<u8> {
    Srgba::new(a[0], a[1], a[2], a[3])
}

#[no_mangle]
pub unsafe extern "C" fn dume_render(ctx: *mut DumeCtx) {
    let ctx = unpointer(ctx);

    let frame = ctx
        .swap_chain
        .get_current_frame()
        .expect("failed to get swap chain frame");

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    ctx.canvas.render(
        &ctx.sample_texture,
        &frame.output.view,
        &mut encoder,
        glam::vec2(
            ctx.swap_chain_desc.width as f32,
            ctx.swap_chain_desc.height as f32,
        ),
    );

    ctx.queue.submit(std::iter::once(encoder.finish()));
}

#[no_mangle]
pub unsafe extern "C" fn dume_free(ctx: *mut DumeCtx) {
    drop(Box::from_raw(ctx));
}

fn create_sample_texture(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("sample_textue"),
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        })
        .create_view(&Default::default())
}

fn unpointer<T>(p: *mut T) -> &'static mut T {
    unsafe { &mut *p }
}

fn canvas(ctx: *mut DumeCtx) -> &'static mut Canvas {
    &mut unpointer(ctx).canvas
}
