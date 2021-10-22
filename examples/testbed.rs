use std::{
    iter::{self},
    sync::Arc,
};

use dume::{TextureSetBuilder, SAMPLE_COUNT, TARGET_FORMAT};
use glam::vec2;
use pollster::block_on;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let adapter = block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = block_on(adapter.request_device(&Default::default(), None)).unwrap();

    let device = Arc::new(device);
    let queue = Arc::new(queue);
    let context = dume::Context::new(Arc::clone(&device), Arc::clone(&queue));

    let mut texture_set_a = context.create_texture_set_builder();
    load_texture(&mut texture_set_a, "assets/image1.jpg", "image1");
    load_texture(&mut texture_set_a, "assets/image2.jpg", "image2");
    context.add_texture_set(texture_set_a.build(256, 8192).unwrap());

    let mut texture_set_b = context.create_texture_set_builder();
    load_texture(&mut texture_set_b, "assets/image3.jpg", "image3");
    context.add_texture_set(texture_set_b.build(256, 4096).unwrap());

    let image1 = context.texture_for_name("image1").unwrap();
    let image2 = context.texture_for_name("image2").unwrap();
    let image3 = context.texture_for_name("image3").unwrap();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    let surface = unsafe { instance.create_surface(&window) };
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: TARGET_FORMAT,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Immediate,
    };
    surface.configure(&device, &surface_config);
    let mut sample_texture = create_sample_texture(&device, &surface_config);

    let mut canvas = context.create_canvas(vec2(
        window.inner_size().to_logical(window.scale_factor()).width,
        window.inner_size().to_logical(window.scale_factor()).height,
    ));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    surface_config.width = new_size.width;
                    surface_config.height = new_size.height;
                    surface.configure(&device, &surface_config);
                    sample_texture = create_sample_texture(&device, &surface_config);
                }

                _ => {}
            },
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                canvas.draw_sprite(image1, vec2(0., 0.), 500.);
                canvas.draw_sprite(image3, vec2(100., 100.), 400.);
                canvas.draw_sprite(image2, vec2(0., 0.), 500.);

                let mut encoder = device.create_command_encoder(&Default::default());
                let frame = surface.get_current_texture().unwrap();
                canvas.render(
                    &mut encoder,
                    &frame.texture.create_view(&Default::default()),
                    &sample_texture.create_view(&Default::default()),
                );

                queue.submit(iter::once(encoder.finish()));

                frame.present();
            }
            _ => {}
        }
    });
}

fn create_sample_texture(
    device: &wgpu::Device,
    surface_config: &wgpu::SurfaceConfiguration,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sample_texture"),
        size: wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    })
}

fn load_texture(builder: &mut TextureSetBuilder, path: &str, id: &str) {
    let image = image::open(path).unwrap().to_rgba8();
    let width = image.width();
    let height = image.height();
    let mut image = image.into_raw();
    dume::convert_rgba_to_bgra(&mut image);
    builder.add_texture(width, height, image, id);
}
