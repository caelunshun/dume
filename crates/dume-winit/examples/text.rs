//! Renders a bunch of text.

use instant::Instant;

use dume::{Canvas, Context, TextBlob};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use winit::{event_loop::EventLoop, window::WindowBuilder};

struct App {
    text: TextBlob,
    start_time: Instant,
}

impl App {
    pub fn new(cx: &Context) -> Self {
        cx.add_font(include_bytes!("../../../assets/ZenAntiqueSoft-Regular.ttf").to_vec())
            .unwrap();
        cx.add_font(include_bytes!("../../../assets/Allison-Regular.ttf").to_vec())
            .unwrap();
        cx.set_default_font_family("Zen Antique Soft");

        let text = cx.create_text_blob(
            dume::text!("@size[50][@color[0,0,0][Dume can render text.] @color[200,30,50][Here is some in scarlet.] @font[Allison][Here's a different font.]]"),
            Default::default(),
        );
        Self {
            text,
            start_time: Instant::now(),
        }
    }
}

impl Application for App {
    fn draw(&mut self, canvas: &mut Canvas) {
        let size = canvas.size();
        canvas
            .begin_path()
            .rect(Vec2::ZERO, size)
            .solid_color((255, 255, 255, 255))
            .fill();

        canvas
            .context()
            .resize_text_blob(&mut self.text, canvas.size());
        canvas.draw_text(&self.text, vec2(10., 50.), 1.);

        let elapsed = self.start_time.elapsed().as_secs();
        let counter_text = canvas.context().create_text_blob(
            dume::text!("@size[12][{} seconds]", elapsed),
            Default::default(),
        );
        let time = self.start_time.elapsed().as_secs_f32();
        let offset = (time.sin() / 2. + 1.) / 2. * 100.;
        canvas.translate(Vec2::splat(offset));
        canvas.draw_text(&counter_text, vec2(10., 500.), 1.);
        canvas.reset_transform();

        let pos = Vec2::splat((time.sin() + 1.) / 2. * 500.);
        canvas
            .begin_path()
            .translate(pos)
            .rounded_rect(Vec2::ZERO, Vec2::splat(200.), 5.)
            .linear_gradient(pos, pos + 200., (0, 0, 0, u8::MAX), (200, 30, 60, u8::MAX))
            .fill();
        canvas
            .solid_color((0, 0, 0, u8::MAX))
            .stroke_width(2.)
            .stroke();

        canvas.reset_transform();
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Error).init().unwrap();
    }
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Dume Text Example")
        .build(&event_loop)
        .unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;

        let app = App::new(dume.context());

        dume.run(event_loop, app);
    });
}
