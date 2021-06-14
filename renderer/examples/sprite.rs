use std::{fs, iter, sync::Arc};

use dume_renderer::{
    markup, Align, Baseline, Canvas, SpriteData, SpriteDescriptor, TextLayout, TextStyle,
    TARGET_FORMAT,
};
use glam::{vec2, Vec2};
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
        .with_resizable(false)
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

    let swap_chain = device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: TARGET_FORMAT,
            width: width as u32,
            height: height as u32,
            present_mode: wgpu::PresentMode::Fifo,
        },
    );

    let mut canvas = Canvas::new(Arc::clone(&device), Arc::clone(&queue));

    let sprite1 = canvas.create_sprite(SpriteDescriptor {
        name: "sprite1",
        data: SpriteData::Encoded(
            &fs::read("/home/caelum/dev/riposte/assets/texture/tile/grassland_basecolor.png")
                .unwrap(),
        ),
    });
    const NUM_SPRITES: usize = 2;
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

    let text = markup::parse(
        "My name is @size{40}{@color{rgb(255, 0, 0)}{Ozymandias}}, King of Kings; look on my Works, ye Mighty,@icon{sprite1} and despair!",
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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                for (pos, vel) in &mut sprites {
                    if vel.x.abs() < 0.1 || vel.y.abs() < 0.1 {
                        vel.x = (fastrand::f32() - 0.5) * 0.2;
                        vel.y = (fastrand::f32() - 0.5) * 0.2;
                    }
                    *pos += *vel;
                    *vel *= 0.99;
                }

                for (pos, _) in &sprites {
                    canvas.draw_sprite(sprite1, *pos, 50.0);
                }

                canvas.draw_paragraph(vec2(200.0, 200.0), &paragraph);

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                let frame = swap_chain.get_current_frame().unwrap();
                canvas.render(
                    &frame.output.view,
                    &mut encoder,
                    Vec2::new(width as f32, height as f32),
                );
                queue.submit(iter::once(encoder.finish()));
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
