//! Renders a bunch of text.

use std::time::Instant;

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
            dume::text!("@size[60][{} seconds]", elapsed),
            Default::default(),
        );
        canvas.draw_text(&counter_text, vec2(10., 500.), 1.);
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
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
