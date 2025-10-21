extern crate sdl2;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod apu;
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

// --- Import audio and other necessary components ---
use sdl2::audio::{AudioSpecDesired};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// --- Audio Constants ---
const AUDIO_SAMPLE_RATE: i32 = 44100;
const AUDIO_BUFFER_SIZE: u16 = 1024;

fn main() {
    // --- Init SDL2 ---
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let window = video_subsystem
        .window("JazzNess Emulator", 256 * 2, 240 * 2)
        .position_centered()
        .build()
        .unwrap();

    // --- Wrap SDL components for sharing ---
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

    // --- Setup Audio Queue ---
    let desired_spec = AudioSpecDesired {
        freq: Some(AUDIO_SAMPLE_RATE),
        channels: Some(1), // mono
        samples: Some(AUDIO_BUFFER_SIZE),
    };
    
    let audio_queue = Rc::new(RefCell::new(
        audio_subsystem.open_queue::<f32, _>(None, &desired_spec).unwrap()
    ));
    audio_queue.borrow().resume(); // Start playing audio

    // --- Key Mapping ---
    let mut key_map = HashMap::new();
    key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_A);
    key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_B);
    key_map.insert(Keycode::Backspace, joypad::JoypadButton::SELECT);
    key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);

    // --- Load ROM ---
    let mut file = File::open("Mario.nes").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let rom = Rom::new(&buffer).unwrap();

    // --- Game Loop Closure ---
    let frame = Rc::new(RefCell::new(Frame::new()));
    let target_frame_time = Duration::from_millis(1000 / 60);

    let canvas_clone = Rc::clone(&canvas);
    let texture_clone = Rc::clone(&texture);
    let frame_clone = Rc::clone(&frame);
    let audio_queue_clone = Rc::clone(&audio_queue);

    let game_loop = move |ppu: &ppu::NesPPU, _joypad: &mut joypad::Joypad, apu: &mut apu::Apu| {
        let frame_start_time = Instant::now();

        // Render video
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

        // Queue audio
        let audio_samples = apu.take_samples();
        if !audio_samples.is_empty() {
             if audio_queue_clone.borrow().size() > (AUDIO_BUFFER_SIZE * 2) as u32 {
                 audio_queue_clone.borrow().clear();
             }
             audio_queue_clone.borrow().queue(&audio_samples);
        }

        // Sleep to maintain frame rate
        let elapsed_time = frame_start_time.elapsed();
        if elapsed_time < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed_time);
        }
    };

    // --- Create Bus and CPU ---
    let bus = Bus::new(rom, game_loop);
    let mut cpu = CPU::new(bus);
    cpu.reset();

    // --- Emulator Run Loop ---
    let instruction_counter = Cell::new(0u32);
    cpu.run_with_callback(move |cpu| {
        // Throttling logic for handling input events
        let count = instruction_counter.get();
        instruction_counter.set(count + 1);
        if count < 1000 {
            return;
        }
        instruction_counter.set(0);

        // Handle SDL events
        for event in event_pump.borrow_mut().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape), ..
                } => std::process::exit(0),

                Event::KeyDown { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = key_map.get(&keycode) {
                            cpu.bus.joypad1.set_button_pressed_status(*button, true);
                        }
                    }
                }

                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = key_map.get(&keycode) {
                            cpu.bus.joypad1.set_button_pressed_status(*button, false);
                        }
                    }
                }
                _ => {}
            }
        }
    });
}

