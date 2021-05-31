use std::{fs, iter, sync::Arc, time::Instant};

use dume_renderer::{Canvas, SpriteData, SpriteDescriptor, TARGET_FORMAT};
use glam::Vec2;
use minifb::Window;
use pollster::block_on;

fn main() {
    let width = 1920 / 2;
    let height = 1080 / 2;
    let mut window = Window::new("Dume", width, height, Default::default()).unwrap();

    let instance = wgpu::Instance::new(wgpu::BackendBit::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .expect("failed to find adapter");
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
        data: SpriteData::Encoded(&fs::read("/home/caelum/Pictures/test.jpg").unwrap()),
    });
    let sprite2 = canvas.create_sprite(SpriteDescriptor {
        name: "sprite2",
        data:  SpriteData::Encoded(&fs::read("/home/caelum/Pictures/volume1.png").unwrap()),
    });

    let start = Instant::now();
    while window.is_open() {
        let time = start.elapsed().as_secs_f32();
        let pos = Vec2::new(time.sin() * 100.0 + width as f32 / 2.0, 50.0);
        canvas.draw_sprite(sprite1, pos, 500.0);

        canvas.draw_sprite(sprite2, Vec2::ZERO, 600.0);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let frame = swap_chain.get_current_frame().unwrap();
        canvas.render(
            &frame.output.view,
            &mut encoder,
            Vec2::new(width as f32, height as f32),
        );
        queue.submit(iter::once(encoder.finish()));

        window.update();
    }
}
