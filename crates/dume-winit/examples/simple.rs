use std::time::Instant;

use dume::{Canvas, Srgba};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use winit::{event_loop::EventLoop, window::Window};

struct App {
    start: Instant,
}

impl Application for App {
    fn draw(&mut self, canvas: &mut Canvas) {
        let _time = self.start.elapsed().as_secs_f32();
        canvas
            .solid_color(Srgba::new(230, 30, 80, u8::MAX))
            .begin_path()
            .move_to(Vec2::splat(10.))
            .line_to(vec2(100., 20.))
            .line_to(vec2(110., 120.))
            .line_to(vec2(10., 110.))
            .line_to(Vec2::splat(10.))
            .fill();
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;
        let app = App {
            start: Instant::now(),
        };
        dume.run(event_loop, app);
    });
}
