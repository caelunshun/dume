//! FFI bindings to `winit`.

use std::{
    ffi::{c_void, CStr},
    os::raw::c_char,
    ptr,
    time::Instant,
};

use once_cell::sync::Lazy;
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, ModifiersState, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[repr(C)]
pub struct WindowOptions {
    pub title: *const c_char,
    pub width: u32,
    pub height: u32,
}

#[no_mangle]
pub unsafe extern "C" fn winit_event_loop_new() -> *mut EventLoop<()> {
    Lazy::force(&STARTUP_TIME);
    Box::leak(Box::new(EventLoop::new())) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn winit_window_new(
    options: &WindowOptions,
    event_loop: *const EventLoop<()>,
) -> *mut Window {
    let window = WindowBuilder::new()
        .with_title(
            CStr::from_ptr(options.title)
                .to_str()
                .expect("invalid UTF-8 in window title"),
        )
        .with_inner_size(LogicalSize::new(options.width, options.height))
        .build(&*event_loop)
        .expect("failed to create window");
    Box::leak(Box::new(window)) as *mut _
}

#[repr(C)]
pub struct Event {
    pub kind: EventKind,
    pub data: EventData,
}

#[repr(C)]
pub enum EventKind {
    CloseRequested,
    RedrawRequested,
    MainEventsCleared,
    Resized,
    Character,
    Keyboard,
    Mouse,
    CursorMove,
    Scroll,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum Action {
    Press,
    Release,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Modifiers {
    control: bool,
    alt: bool,
    shift: bool,
}

impl From<ModifiersState> for Modifiers {
    fn from(m: ModifiersState) -> Self {
        Modifiers {
            control: m.contains(ModifiersState::CTRL),
            alt: m.contains(ModifiersState::ALT),
            shift: m.contains(ModifiersState::SHIFT),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyboardEvent {
    pub key: u32,
    pub action: Action,
    pub modifiers: Modifiers,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MouseEvent {
    pub mouse: u32,
    pub action: Action,
    pub modifiers: Modifiers,
}

#[repr(C)]
pub union EventData {
    pub empty: u8,
    pub new_size: [u32; 2],
    pub c: u32,
    pub keyboard: KeyboardEvent,
    pub mouse: MouseEvent,
    pub cursor_pos: [f32; 2],
    pub scroll_delta: [f32; 2],
}

impl EventData {
    pub fn empty() -> Self {
        Self { empty: 0 }
    }
}

#[repr(C)]
pub enum CControlFlow {
    Poll,
    Exit,
}

impl From<CControlFlow> for ControlFlow {
    fn from(c: CControlFlow) -> Self {
        match c {
            CControlFlow::Poll => ControlFlow::Poll,
            CControlFlow::Exit => ControlFlow::Exit,
        }
    }
}

type WinitEvent<'a> = winit::event::Event<'a, ()>;

#[no_mangle]
pub unsafe extern "C" fn winit_window_request_redraw(window: *const Window) {
    (*window).request_redraw()
}

#[no_mangle]
pub unsafe extern "C" fn winit_window_set_cursor_pos(window: *const Window, x: f32, y: f32) {
    (*window)
        .set_cursor_position(LogicalPosition::new(x, y))
        .expect("failed to set cursor position");
}

#[no_mangle]
pub unsafe extern "C" fn winit_window_grab_cursor(window: *const Window, grabbed: bool) {
    (*window)
        .set_cursor_grab(grabbed)
        .expect("failed to grab cursor");
}

#[no_mangle]
pub unsafe extern "C" fn winit_event_loop_run(
    event_loop: *mut EventLoop<()>,
    callback: extern "C" fn(*mut c_void, Event) -> CControlFlow,
    userdata: *mut c_void,
) {
    let mut modifiers = ModifiersState::default();

    let event_loop = ptr::read(event_loop);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        let (kind, data) = match event {
            WinitEvent::RedrawRequested(_) => (EventKind::RedrawRequested, EventData::empty()),
            WinitEvent::MainEventsCleared => (EventKind::MainEventsCleared, EventData::empty()),
            WinitEvent::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => (
                    EventKind::Resized,
                    EventData {
                        new_size: [new_size.width, new_size.height],
                    },
                ),

                WindowEvent::CloseRequested => (EventKind::CloseRequested, EventData::empty()),

                WindowEvent::ReceivedCharacter(c) => {
                    (EventKind::Character, EventData { c: c as u32 })
                }

                WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => (
                    EventKind::Keyboard,
                    EventData {
                        keyboard: KeyboardEvent {
                            key: input.virtual_keycode.map(|k| k as u32).unwrap_or_default(),
                            action: match input.state {
                                ElementState::Pressed => Action::Press,
                                ElementState::Released => Action::Release,
                            },
                            modifiers: modifiers.into(),
                        },
                    },
                ),
                WindowEvent::ModifiersChanged(m) => {
                    modifiers = m;
                    return;
                }
                WindowEvent::CursorMoved { position, .. } => (
                    EventKind::CursorMove,
                    EventData {
                        cursor_pos: [position.x as f32, position.y as f32],
                    },
                ),
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (
                        EventKind::Scroll,
                        EventData {
                            scroll_delta: [x, y],
                        },
                    ),
                    winit::event::MouseScrollDelta::PixelDelta(delta) => (
                        EventKind::Scroll,
                        EventData {
                            scroll_delta: [delta.x as f32, delta.y as f32],
                        },
                    ),
                },
                WindowEvent::MouseInput { state, button, .. } => (
                    EventKind::Mouse,
                    EventData {
                        mouse: MouseEvent {
                            mouse: match button {
                                MouseButton::Left => 0,
                                MouseButton::Right => 1,
                                MouseButton::Middle => 2,
                                MouseButton::Other(_) => 0,
                            },
                            action: match state {
                                ElementState::Pressed => Action::Press,
                                ElementState::Released => Action::Release,
                            },
                            modifiers: modifiers.into(),
                        },
                    },
                ),
                _ => return,
            },
            _ => {
                return;
            }
        };

        *control_flow = callback(userdata, Event { kind, data }).into();
    });
}

#[no_mangle]
pub unsafe extern "C" fn winit_window_free(window: *mut Window) {
    drop(Box::from_raw(window));
}

static STARTUP_TIME: Lazy<Instant> = Lazy::new(Instant::now);

#[no_mangle]
pub unsafe extern "C" fn winit_get_time() -> f64 {
    Instant::now().duration_since(*STARTUP_TIME).as_secs_f64()
}
