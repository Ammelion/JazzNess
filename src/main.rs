extern crate sdl2;
use std::collections::HashMap;
use std::time::{Duration, Instant}; // Added for frame limiting

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

fn main() {
    // --- Init SDL2 ---
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("JazzNess Emulator", 256 * 2, 240 * 2)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

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
    let mut file = File::open("donkeykong.nes").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let rom = Rom::new(&buffer).unwrap();

    // --- Set up the Game Loop Closure ---
    let mut frame = Frame::new();
    
    // NEW: Define the target time for one frame (1000ms / 60fps)
    let target_frame_time = Duration::from_millis(1000 / 60);

    let game_loop = move |ppu: &ppu::NesPPU, joypad: &mut joypad::Joypad| {
        // NEW: Get the time at the start of the frame
        let frame_start_time = Instant::now();

        render::render(ppu, &mut frame);
        texture
            .update(None, &frame.data, Frame::WIDTH * 3)
            .unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),

                Event::KeyDown { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = key_map.get(&keycode) {
                            joypad.set_button_pressed_status(*button, true);
                        }
                    }
                }
                
                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = key_map.get(&keycode) {
                            joypad.set_button_pressed_status(*button, false);
                        }
                    }
                }
                _ => {}
            }
        }

        // NEW: Calculate how long the frame took and sleep if we are too fast
        let elapsed_time = frame_start_time.elapsed();
        if elapsed_time < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed_time);
        }
    };

    // --- Create the Bus and CPU, then run the emulator ---
    let bus = Bus::new(rom, game_loop);
    let mut cpu = CPU::new(bus);
    cpu.reset();
    
    cpu.run_with_callback(|_| {});
}