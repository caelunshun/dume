use std::{fs, iter, sync::Arc, time::Instant};

use dume_renderer::{Canvas, SpriteData, SpriteDescriptor, TARGET_FORMAT};
use glam::Vec2;
use pollster::block_on;
use simple_logger::SimpleLogger;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    SimpleLogger::new().with_level(log::LevelFilter::Warn).init().unwrap();
    
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
        data: SpriteData::Encoded(&fs::read("/home/caelum/Pictures/test.png").unwrap()),
    });
    let sprite2 = canvas.create_sprite(SpriteDescriptor {
        name: "sprite2",
        data: SpriteData::Encoded(&fs::read("/home/caelum/Pictures/volume1.png").unwrap()),
    });

    let start = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let time = start.elapsed().as_secs_f32();
                let pos = Vec2::new(time.sin() * 100.0 + width as f32 / 2.0, 50.0);
                canvas.draw_sprite(sprite1, pos, 500.0);

                canvas.draw_sprite(sprite2, Vec2::ZERO, 600.0);

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
