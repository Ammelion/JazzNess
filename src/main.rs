// in src/main.rs

extern crate sdl2;
pub mod bus;
pub mod cartridge;
pub mod cpu;
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
    // --- 1. Init SDL2 ---
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

    // --- 2. Load the ROM ---
    let mut file = File::open("pacman.nes").unwrap(); // Make sure pacman.nes is in your project root
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let rom = Rom::new(&buffer).unwrap();

    // --- 3. Set up the Game Loop Closure ---
    let mut frame = Frame::new();
    let game_loop = move |ppu: &ppu::NesPPU| {
        // This is our main loop, it runs once per frame
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
                _ => { /* Handle controller input here later */ }
            }
        }
    };

    // --- 4. Create the Bus and CPU, then run the emulator ---
    let bus = Bus::new(rom, game_loop);
    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.run_with_callback(|_| {}); // Start the CPU emulation
}