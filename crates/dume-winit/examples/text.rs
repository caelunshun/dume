//! Renders a bunch of text.

use instant::Instant;

use dume::{Canvas, Context, TextBlob};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use winit::{event_loop::EventLoop, window::WindowBuilder};

struct App {
    text: TextBlob,
    text2: TextBlob,
    start_time: Instant,
}

impl App {
    pub fn new(cx: &Context) -> Self {
        cx.add_font(include_bytes!("../../../assets/ZenAntiqueSoft-Regular.ttf").to_vec())
            .unwrap();
        cx.add_font(include_bytes!("../../../assets/Allison-Regular.ttf").to_vec())
            .unwrap();
        cx.set_default_font_family("Zen Antique Soft");

        let text = dume::text!("@size[50][@color[0,0,0][Dume can render text.] @color[200,30,50][Here is some in scarlet.] @font[Allison][Here's a different font.]]");
        let text2 = dume::text!(
            "@color[0,0,0][I met a traveller from an antique land,
            Who said—“Two vast and trunkless legs of stone
            Stand in the desert.... Near them, on the sand,
            Half sunk a shattered visage lies, whose frown,
            And wrinkled lip, and sneer of cold command,
            Tell that its sculptor well those passions read
            Which yet survive, stamped on these lifeless things,
            The hand that mocked them, and the heart that fed;
            And on the pedestal, these words appear:
            My name is Ozymandias, King of Kings;
            Look on my Works, ye Mighty, and despair!
            Nothing beside remains. Round the decay
            Of that colossal Wreck, boundless and bare
            The lone and level sands stretch far away.]"
        );
        let text = cx.create_text_blob(text, Default::default());
        let text2 = cx.create_text_blob(text2, Default::default());
        Self {
            text,
            text2,
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
        canvas.draw_text(&self.text2, vec2(10., 200.), 1.);

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
            .linear_gradient(Vec2::ZERO, Vec2::splat(200.), (0, 0, 0, u8::MAX), (200, 30, 60, u8::MAX))
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
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Error)
            .init()
            .unwrap();
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
