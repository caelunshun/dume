use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use zylo::{Canvas, Color, Context, Vec2};

use crate::{select_optimal_backend, Presenter};

pub trait Render {
    fn render(&mut self, canvas: &mut Canvas, target_size: Vec2);
}

pub fn run(
    build_window: impl FnOnce(&EventLoop<()>) -> Window,
    mut renderer: impl Render + 'static,
) {
    let event_loop = EventLoop::new();
    let window = build_window(&event_loop);

    let backend = select_optimal_backend();
    let mut context = Context::new(backend);

    let mut presenter = Presenter::new(window, &context);

    let window = presenter.window();
    let mut layer = context.create_layer(
        window.inner_size().width,
        window.inner_size().height,
        window.scale_factor() as f32,
    );

    let mut canvas = Canvas::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                layer = context.create_layer(
                    new_size.width,
                    new_size.height,
                    presenter.window().scale_factor() as f32,
                );
            }
            Event::MainEventsCleared => presenter.window().request_redraw(),
            Event::RedrawRequested(_) => {
                renderer.render(&mut canvas, layer.logical_size());
                layer.fill(Color::WHITE);
                canvas.render_to_layer(&mut context, &mut layer);
                presenter.present(&layer);
            }
            _ => {}
        }
    });
}
