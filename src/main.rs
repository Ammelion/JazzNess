// In src/main.rs

extern crate sdl2;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod joypad;
pub mod palette;
pub mod ppu;
pub mod render;

use bus::Bus;
use cartridge::Rom;
use cpu::CPU;
use render::frame::Frame;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::fs::File;
use std::io::Read;

// --- Import Rc and RefCell ---
use std::cell::{Cell, RefCell}; // Cell is still needed for the closure
use std::rc::Rc;

fn main() {
    // --- Init SDL2 ---
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("JazzNess Emulator", 256 * 2, 240 * 2)
        .position_centered()
        .build()
        .unwrap();

    // --- Wrap SDL components in Rc<RefCell<...>> ---
    let canvas = Rc::new(RefCell::new(
        window.into_canvas().present_vsync().build().unwrap(),
    ));
    let mut event_pump = Rc::new(RefCell::new(sdl_context.event_pump().unwrap()));
    let texture_creator = canvas.borrow().texture_creator();
    let texture = Rc::new(RefCell::new(
        texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
            .unwrap(),
    ));

    // --- Create a key mapping ---
    let mut key_map = HashMap::new();
    key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_A);
    key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_B);
    key_map.insert(Keycode::Backspace, joypad::JoypadButton::SELECT);
    key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);

    // --- Load the ROM ---
    // Make sure to change this to the ROM you want to run!
    let mut file = File::open("Mario.nes").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let rom = Rom::new(&buffer).unwrap();

    // --- Set up the Game Loop Closure ---
    let frame = Rc::new(RefCell::new(Frame::new()));
    let target_frame_time = Duration::from_millis(1000 / 60);

    let canvas_clone = Rc::clone(&canvas);
    let texture_clone = Rc::clone(&texture);
    let frame_clone = Rc::clone(&frame);

    // --- Note: Fix unused variable warning by prefixing joypad with _ ---
    let game_loop = move |ppu: &ppu::NesPPU, _joypad: &mut joypad::Joypad| {
        let frame_start_time = Instant::now();

        render::render(ppu, &mut frame_clone.borrow_mut());
        texture_clone
            .borrow_mut()
            .update(None, &frame_clone.borrow().data, Frame::WIDTH * 3)
            .unwrap();
        canvas_clone
            .borrow_mut()
            .copy(&texture_clone.borrow(), None, None)
            .unwrap();
        canvas_clone.borrow_mut().present();

        let elapsed_time = frame_start_time.elapsed();
        if elapsed_time < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed_time);
        }
    };

    // --- Create the Bus and CPU ---
    let bus = Bus::new(rom, game_loop);
    let mut cpu = CPU::new(bus);
    cpu.reset();

    // --- Add a counter to throttle event polling ---
    let instruction_counter = Cell::new(0u32);

    // --- Run the emulator ---
    // The callback now just handles input and throttling
    cpu.run_with_callback(move |cpu| {

        // --- Throttling Logic ---
        let count = instruction_counter.get();
        instruction_counter.set(count + 1);
        if count < 1000 {
            // Only poll events every 1000 instructions
            return;
        }
        instruction_counter.set(0); // Reset counter
        // --- End of Throttling Logic ---

        // This code block now only runs periodically
        for event in event_pump.borrow_mut().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape), ..
                } => std::process::exit(0),

                Event::KeyDown { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = key_map.get(&keycode) {
                            // Make sure joypad1 is public in bus.rs
                            cpu.bus.joypad1.set_button_pressed_status(*button, true);
                        }
                    }
                }

                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = key_map.get(&keycode) {
                            // Make sure joypad1 is public in bus.rs
                            cpu.bus.joypad1.set_button_pressed_status(*button, false);
                        }
                    }
                }
                _ => {}
            }
        }
    });
}