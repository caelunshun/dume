//! Renders a bunch of text.

use instant::Instant;

use dume::{Canvas, Context, TextBlob};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

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
        let time = self.start_time.elapsed().as_secs_f32();
        let size = canvas.size();

        // Background
        canvas
            .linear_gradient(
                Vec2::ZERO,
                vec2(size.x, 0.),
                (255, 200, 30, 255),
                (255, 255, 255, 255),
            )
            .fill_rect(Vec2::ZERO, size);
        canvas
            .begin_path()
            .move_to(vec2(1000., 0.))
            .line_to(vec2(1400., 1080.))
            .solid_color((0, 0, 0, u8::MAX))
            .stroke_width(10.)
            .stroke();

        canvas
            .context()
            .resize_text_blob(&mut self.text, canvas.size());

        canvas.draw_text(&self.text, vec2(10., 50.), 1.);
        canvas.draw_text(&self.text2, vec2(10., 200.), 1.);

        let pos = Vec2::splat((time.sin() + 1.) / 2. * 500.);
        canvas
            .radial_gradient(
                pos + 100.,
                100.,
                (227, 101, 105, u8::MAX),
                (151, 146, 216, 50),
            )
            .fill_rect(pos, Vec2::splat(200.))
            .solid_color((0, 0, 0, u8::MAX))
            .begin_path()
            .rect(pos, Vec2::splat(200.))
            .stroke_width(5.)
            .stroke();
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Dume Text Example")
        .with_inner_size(LogicalSize::new(1920, 1080))
        .build(&event_loop)
        .unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;

        let app = App::new(dume.context());

        dume.run(event_loop, app);
    });
}
