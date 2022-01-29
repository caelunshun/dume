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
        let time = self.start.elapsed().as_secs_f32();
        canvas
            .solid_color(Srgba::new(230, 30, 80, u8::MAX))
            .fill_rect(vec2(50., 50.), Vec2::splat(1000.))
            .solid_color(Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX))
            .fill_circle(
                vec2(300., 300.).lerp(vec2(500., 500.), (time.sin() + 1.) / 2.),
                250.,
            );
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
