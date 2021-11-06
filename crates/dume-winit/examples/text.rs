//! Renders a bunch of text.

use std::iter;

use dume::{Canvas, Context, Text, TextBlob, TextSection, TextStyle};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use winit::{event_loop::EventLoop, window::WindowBuilder};

static TEXT: &str = r#"
The spotted hawk swoops by and accuses me, he complains of my gab and my loitering.
I too am not a bit tamed, I too am untranslatable,
I sound my barbaric yawp over the roofs of the world.
The last scud of day holds back for me,
It flings my likeness after the rest and true as any on the shadow’d wilds,
It coaxes me to the vapor and the dusk.
I depart as air, I shake my white locks at the runaway sun,
I effuse my flesh in eddies, and drift it in lacy jags.
I bequeath myself to the dirt to grow from the grass I love,
If you want me again look for me under your boot-soles.
You will hardly know who I am or what I mean,
But I shall be good health to you nevertheless,
And filter and fibre your blood.
Failing to fetch me at first keep encouraged,
Missing me one place search another,
I stop somewhere waiting for you.
"#;

struct App {
    text: TextBlob,
}

impl App {
    pub fn new(cx: &Context) -> Self {
        cx.add_font(include_bytes!("../../../assets/ZenAntiqueSoft-Regular.ttf").to_vec())
            .unwrap();
        cx.set_default_font_family("Zen Antique Soft");

        let  text = cx.create_text_blob(
            Text::from_sections(iter::once(TextSection::Text {
                text: TEXT.into(),
                style: TextStyle {
                    size: 20.,
                    color: (0, 0, 0, 255).into(),
                    ..Default::default()
                },
            })),
            Default::default(),
        );
        Self { text }
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
