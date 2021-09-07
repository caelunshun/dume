use std::{fs, iter, sync::Arc};

use dume_renderer::{
    markup, Align, Baseline, Canvas, Rect, SpriteData, SpriteDescriptor, TextLayout, TextStyle,
    SAMPLE_COUNT, TARGET_FORMAT,
};
use glam::{vec2, Vec2};
use palette::Srgba;
use pollster::block_on;
use simple_logger::SimpleLogger;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()
        .unwrap();

    let width = 1920 / 2;
    let height = 1080 / 2;
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Dume")
        .with_inner_size(LogicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .expect("failed to find adapter");
    println!("Adapter: {:?}", adapter.get_info());
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: Default::default(),
            limits: Default::default(),
        },
        None,
    ))
    .unwrap();
    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: TARGET_FORMAT,
        width: width as u32,
        height: height as u32,
        present_mode: wgpu::PresentMode::Fifo,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
    let sample_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sample"),
        size: wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
    });
    let mut sample_texture = sample_texture.create_view(&Default::default());

    let mut canvas = Canvas::new(Arc::clone(&device), Arc::clone(&queue));

    let sprite1 = canvas.create_sprite(SpriteDescriptor {
        name: "sprite1",
        data: SpriteData::Encoded(
            &fs::read("/home/caelum/dev/riposte/assets/texture/tile/grassland_basecolor.png")
                .unwrap(),
        ),
    });
    const NUM_SPRITES: usize = 512;
    let mut sprites: Vec<_> = iter::repeat_with(|| {
        (
            Vec2::new(
                fastrand::f32() * width as f32,
                fastrand::f32() * height as f32,
            ),
            Vec2::ZERO,
        )
    })
    .take(NUM_SPRITES)
    .collect();

    canvas.load_font(
        fs::read("/home/caelum/dev/riposte/assets/font/Merriweather-Regular.ttf").unwrap(),
    );
    canvas.load_font(fs::read("/home/caelum/Downloads/Merriweather-Bold.ttf").unwrap());
    canvas.load_font(fs::read("/home/caelum/Downloads/Merriweather-Italic.ttf").unwrap());

    let text = markup::parse(
        "@color{rgb(0,142,170)}{My name is @bold{@size{40}{@color{rgb(239,106,0)}{Ozymandias,}}} King of Kings;} look on my @bold{Works}, ye Mighty,@icon{sprite1} and despair!",
        TextStyle::default(),
        |_| String::new(),
    )
    .unwrap();
    let paragraph = canvas.create_paragraph(
        text,
        TextLayout {
            max_dimensions: vec2(600.0, 400.0),
            line_breaks: true,
            baseline: Baseline::Alphabetic,
            align_h: Align::Start,
            align_v: Align::Start,
        },
    );

    let text2 = markup::parse(
        "@size{30}{Some text that will\nbe cut out....}",
        TextStyle::default(),
        |_| String::new(),
    )
    .unwrap();
    let paragraph2 = canvas.create_paragraph(
        text2,
        TextLayout {
            max_dimensions: vec2(600.0, 400.0),
            line_breaks: true,
            baseline: Baseline::Top,
            align_h: Align::Start,
            align_v: Align::Start,
        },
    );

    let text3 = markup::parse(
        "@italic{@size{100}{@color{rgb(192, 78, 32)}{DU}@color{rgb(78, 192, 32)}{ME}}}",
        TextStyle::default(),
        |_| String::new(),
    )
    .unwrap();
    let paragraph3 = canvas.create_paragraph(
        text3,
        TextLayout {
            max_dimensions: vec2(width as f32, height as f32),
            line_breaks: true,
            baseline: Baseline::Top,
            align_h: Align::Center,
            align_v: Align::Center,
        },
    );

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                for (pos, vel) in &mut sprites {
                    if vel.x.abs() < 0.1 || vel.y.abs() < 0.1 {
                        vel.x = (fastrand::f32() - 0.5) * 1.1;
                        vel.y = (fastrand::f32() - 0.5) * 1.1;
                    }
                    *pos += *vel;
                    *vel *= 0.999;
                }

                for (pos, _) in &sprites {
                    canvas.draw_sprite(sprite1, *pos, 50.0);
                }

                canvas.draw_paragraph(vec2(200.0, 200.0), &paragraph);

                canvas
                    .scissor_rect(Rect {
                        pos: vec2(230.0, 230.0),
                        size: vec2(300.0, 65.0),
                    })
                    .draw_paragraph(vec2(230.0, 230.0), &paragraph2)
                    .clear_scissor();

                canvas
                    .begin_path()
                    .move_to(vec2(100.0, 100.0))
                    .line_to(vec2(150.0, 150.0))
                    .quad_to(vec2(250.0, 300.0), vec2(400.0, 150.0))
                    .stroke_width(20.0)
                    .solid_color(Srgba::new(8, 127, 226, 128))
                    .stroke();

                canvas
                    .begin_path()
                    .move_to(vec2(300.0, 300.0))
                    .line_to(vec2(400.0, 300.0))
                    .line_to(vec2(400.0, 400.0))
                    .linear_gradient(
                        vec2(300.0, 300.0),
                        vec2(400.0, 400.0),
                        Srgba::new(8, 127, 226, u8::MAX),
                        Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX),
                    )
                    .fill();

                canvas.draw_paragraph(Vec2::ZERO, &paragraph3);

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                let frame = swap_chain.get_current_frame().unwrap();
                canvas.render(
                    &sample_texture,
                    &frame.output.view,
                    &mut encoder,
                    Vec2::new(
                        window.inner_size().width as f32,
                        window.inner_size().height as f32,
                    ),
                );
                queue.submit(iter::once(encoder.finish()));
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                sc_desc.width = new_size.width;
                sc_desc.height = new_size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
                sample_texture = device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some("sample"),
                        size: wgpu::Extent3d {
                            width: new_size.width,
                            height: new_size.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: SAMPLE_COUNT,
                        dimension: wgpu::TextureDimension::D2,
                        format: TARGET_FORMAT,
                        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                    })
                    .create_view(&Default::default());
            }
            _ => (),
        }
    });
}
