use std::{future::Future, iter, sync::Arc};

use dume::{Canvas, Context};
use glam::{vec2, Vec2};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct DumeWinit {
    context: Context,
    main_canvas: Canvas,

    sample_texture: wgpu::TextureView,
    surface: wgpu::Surface,

    window: Window,
}

impl DumeWinit {
    /// Creates an app given the window.
    ///
    /// This function initializes `wgpu` state and creates a [`Context`](dume::Context).
    /// For more control over initialization, use [`from_context`].
    ///
    ///
    /// On WebAssembly targets, this also adds the window to the root HTML element.
    pub async fn new(window: Window) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            let canvas = window.canvas();

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let body = document.body().unwrap();

            body.append_child(&canvas)
                .expect("failed to append canvas to HTML body");
        }

        let (context, surface) = init_context(&window).await;

        Self::from_context(context, surface, window)
    }

    /// Creates an `App` from an existing `Context`.
    pub fn from_context(context: Context, surface: wgpu::Surface, window: Window) -> Self {
        let sample_texture = create_sample_texture(window.inner_size(), context.device());
        configure_surface(&surface, context.device(), window.inner_size());

        Self {
            main_canvas: context.create_canvas(logical_size(&window), window.scale_factor() as f32),
            context,
            sample_texture,
            surface,
            window,
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn main_canvas(&mut self) -> &mut Canvas {
        &mut self.main_canvas
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Runs the main event loop.
    ///
    /// Calls `draw` whenever a frame should be drawn.
    /// Calls `on_event` with any window events received from `winit`.
    pub fn run(mut self, event_loop: EventLoop<()>, mut application: impl Application + 'static) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => {
                    let mut encoder = self
                        .context
                        .device()
                        .create_command_encoder(&Default::default());

                    let frame = self
                        .surface
                        .get_current_texture()
                        .expect("failed to get swap chain frame");

                    application.draw(&mut self.main_canvas);

                    self.main_canvas.render(
                        &mut encoder,
                        &frame.texture.create_view(&Default::default()),
                        &self.sample_texture,
                    );

                    self.context.queue().submit(iter::once(encoder.finish()));

                    frame.present();
                }
                Event::WindowEvent { event, .. } => {
                    application.on_event(&self.context, &event);
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(new_size) => {
                            configure_surface(&self.surface, self.context.device(), new_size);
                            self.sample_texture =
                                create_sample_texture(new_size, self.context.device());

                            self.main_canvas.resize(
                                logical_size(&self.window),
                                self.window.scale_factor() as f32,
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        });
    }
}

fn create_sample_texture(size: PhysicalSize<u32>, device: &wgpu::Device) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: dume::SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: dume::TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        })
        .create_view(&Default::default())
}

fn configure_surface(surface: &wgpu::Surface, device: &wgpu::Device, size: PhysicalSize<u32>) {
    surface.configure(
        device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: dume::TARGET_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        },
    )
}

pub trait Application {
    fn draw(&mut self, canvas: &mut Canvas);

    fn on_event(&mut self, context: &Context, event: &WindowEvent) {
        let _ = (context, event);
    }
}

fn logical_size(window: &Window) -> Vec2 {
    let size = window.inner_size().to_logical(window.scale_factor());
    vec2(size.width, size.height)
}

async fn init_context(window: &Window) -> (Context, wgpu::Surface) {
    let (device, queue, surface) = init_wgpu(window).await;
    let device = Arc::new(device);
    let queue = Arc::new(queue);

    (Context::builder(device, queue).build(), surface)
}

async fn init_wgpu(window: &Window) -> (wgpu::Device, wgpu::Queue, wgpu::Surface) {
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("failed to get a suitable adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                #[cfg(target_arch = "wasm32")]
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
                #[cfg(not(target_arch = "wasm32"))]
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("failed to get wgpu device");

    (device, queue, surface)
}

/// Convenience function to block on a future, useful for [`Context::new`].
///
/// This function works on both native and WebAssembly targets.
pub fn block_on(future: impl Future<Output = ()> + 'static) {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(future);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(future);
    }
}
