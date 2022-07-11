use dume::{Align, Canvas, Context, Srgba, TextBlob, TextOptions};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use winit::{event_loop::EventLoop, window::Window};

struct App {
    text: TextBlob,
}

impl App {
    pub fn new(context: &Context) -> Self {
        context
            .add_font(include_bytes!("../../../assets/ZenAntiqueSoft-Regular.ttf").to_vec())
            .unwrap();
        context.set_default_font_family("Zen Antique Soft");
        let contents = r#"
        The Northrop (later Northrop Grumman) B-2 Spirit, also known as the Stealth Bomber, is an American heavy strategic bomber, featuring low observable stealth technology designed for penetrating dense anti-aircraft defenses. Designed during the Cold War, it is a flying wing design with a crew of two.[1][3] The bomber is subsonic and can deploy both conventional and thermonuclear weapons, such as up to eighty 500-pound class (230 kg) Mk 82 JDAM GPS-guided bombs, or sixteen 2,400-pound (1,100 kg) B83 nuclear bombs. The B-2 is the only acknowledged aircraft that can carry large air-to-surface standoff weapons in a stealth configuration.

Development started under the "Advanced Technology Bomber" (ATB) project during the Carter administration; its expected performance was one of the President's reasons for the cancellation of the Mach 2 capable B-1A bomber. The ATB project continued during the Reagan administration, but worries about delays in its introduction led to the reinstatement of the B-1 program. Program costs rose throughout development. Designed and manufactured by Northrop, later Northrop Grumman, the cost of each aircraft averaged US$737 million (in 1997 dollars).[4] Total procurement costs averaged $929 million per aircraft, which includes spare parts, equipment, retrofitting, and software support.[4] The total program cost, which included development, engineering and testing, averaged $2.13 billion per aircraft in 1997.[4]

Because of its considerable capital and operating costs, the project was controversial in the U.S. Congress. The winding-down of the Cold War in the latter portion of the 1980s dramatically reduced the need for the aircraft, which was designed with the intention of penetrating Soviet airspace and attacking high-value targets. During the late 1980s and 1990s, Congress slashed plans to purchase 132 bombers to 21. In 2008, a B-2 was destroyed in a crash shortly after takeoff, though the crew ejected safely.[5] As of 2018, twenty B-2s are in service with the United States Air Force, which plans to operate them until 2032, when the Northrop Grumman B-21 Raider is to replace them.[6]

The B-2 is capable of all-altitude attack missions up to 50,000 feet (15,000 m), with a range of more than 6,000 nautical miles (6,900 mi; 11,000 km) on internal fuel and over 10,000 nautical miles (12,000 mi; 19,000 km) with one midair refueling. It entered service in 1997 as the second aircraft designed to have advanced stealth technology after the Lockheed F-117 Nighthawk attack aircraft. Though designed originally as primarily a nuclear bomber, the B-2 was first used in combat dropping conventional, non-nuclear ordnance in the Kosovo War in 1999. It later served in Iraq, Afghanistan, and Libya.[7]
        "#;
        let text = dume::text!("@size[14][{}]", contents);
        let text = context.create_text_blob(
            text,
            TextOptions {
                align_v: Align::Start,
                ..Default::default()
            },
        );
        dbg!(text.min_content_size(), text.max_content_size());
        Self { text }
    }
}

impl Application for App {
    fn draw(&mut self, canvas: &mut Canvas) {
        let size = canvas.size();
        canvas.context().resize_text_blob(&mut self.text, size);
        canvas
            .rect(Vec2::ZERO, size)
            .solid_color(Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX))
            .fill();
        canvas.draw_text(&self.text, vec2(0., 20.), 1.);
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;
        let app = App::new(dume.context());
        dume.run(event_loop, app);
    });
}
