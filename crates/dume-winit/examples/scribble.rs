use dume::{Canvas, StrokeCap};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use rand::{prelude::StdRng, Rng, SeedableRng};
use winit::{event_loop::EventLoop, window::Window};

struct App {
    seed: u64,
}

impl Application for App {
    fn draw(&mut self, canvas: &mut Canvas) {
        let mut rng = StdRng::seed_from_u64(self.seed);

        let size = canvas.size();

        canvas
            .solid_color((30, 130, 200, u8::MAX))
            .fill_rect(Vec2::ZERO, size);

        canvas.begin_path();
        canvas.move_to(size / 2.);
        for _ in 0..100 {
            let pos = vec2(rng.gen::<f32>() * size.x, rng.gen::<f32>() * size.y);
            canvas.line_to(pos);
        }

        canvas
            .stroke_width(2.)
            .stroke_cap(StrokeCap::Square)
            .solid_color((200, 80, 140, u8::MAX))
            .stroke();
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;
        let app = App {
            seed: rand::random(),
        };
        dume.run(event_loop, app);
    });
}
