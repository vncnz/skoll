use smithay_client_toolkit::{
    environment::SimpleGlobal,
    reexports::protocols::xdg::xdg_wm_base::XdgWmBase,
    window::{Event as WindowEvent, Window, WindowType},
    environment::Environment,
};
use wayland_client::{Display, EventQueue, Main};
use smithay_client_toolkit::reexports::client::{protocol::wl_seat, protocol::wl_keyboard};
use smithay_client_toolkit::keyboard::{KeyEvent, KeyState, RepeatKind};
use smithay_client_toolkit::shm::slot::{Buffer, SlotPool};
use smithay_client_toolkit::output::OutputHandler;

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Connect to Wayland display and set up the event queue
    let display = Display::connect_to_env().expect("Failed to connect to Wayland server");
    let mut event_queue = display.create_event_queue();
    let attached_display = (*display).clone().attach(event_queue.token());

    // Initialize the environment (includes xdg_shell, wl_seat, etc.)
    let environment = Environment::new(&attached_display, &[]).expect("Failed to initialize environment");

    // Create a fullscreen window
    let mut window = Window::new(
        &environment,
        WindowType::Fullscreen,
        None,
        "Fullscreen Launcher",
    )
    .expect("Failed to create fullscreen window");
    
    // Make window fullscreen
    window.set_fullscreen(None);

    // Setup a transparent buffer for the window
    let mut slot_pool = SlotPool::new(1024 * 1024, &attached_display).expect("Failed to create slot pool");
    let buffer = slot_pool
        .create_buffer(1280, 720, 1280 * 4, wayland_client::protocol::wl_shm::Format::Argb8888)
        .expect("Failed to create buffer");
    fill_transparent(&buffer);
    window.surface().attach(Some(&buffer), 0, 0);
    window.surface().commit();

    // Track keyboard events
    let keyboard = environment
        .require_global::<wl_seat::WlSeat>()
        .and_then(|seat| {
            seat.get_keyboard(|keyboard| {
                keyboard.quick_assign(|_, event, _| match event {
                    wl_keyboard::Event::Key { key, state, .. } => {
                        if state == wl_keyboard::KeyState::Pressed {
                            if key == 1 { // Replace with the keycode for your desired key
                                println!("Exiting...");
                                std::process::exit(0);
                            }
                        }
                    }
                    _ => {}
                });
            });
        })
        .expect("Failed to set up keyboard");

    // Event loop
    loop {
        event_queue.dispatch(&mut (), |_, _, _| {}).expect("Event dispatch failed");
    }
}

fn fill_transparent(buffer: &Buffer) {
    let canvas = buffer.canvas();
    for pixel in canvas.chunks_exact_mut(4) {
        pixel[0] = 0; // R
        pixel[1] = 0; // G
        pixel[2] = 0; // B
        pixel[3] = 128; // A (128 = semi-transparent)
    }
}