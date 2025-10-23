// In src/emulator.rs

use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc, Mutex};
use std::io::{self, Write};
use crate::debugger::Breakpoint; 

// --- KEEP ALL IMPORTS ---
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::Read;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::audio::AudioSpecDesired;

use crate::bus::Bus;
use crate::cartridge::Rom;
use crate::cpu::CPU;
use crate::render::frame::Frame;
use crate::render;
use crate::apu;
use crate::ppu;
use crate::joypad;
use crate::gamegenie::GameGenieCode;
use crate::bus::Mem; // <--- FIX 1: ADD THIS IMPORT

// --- (Rest of file is unchanged until the loop) ---

const AUDIO_SAMPLE_RATE: i32 = 44100;
const AUDIO_BUFFER_SIZE: u16 = 1024;

pub enum EmulatorCommand {
    LoadRom(String),
    SetGameGenieCodes(Vec<GameGenieCode>),
    Pause, 
}

pub fn run_emulator(rx: mpsc::Receiver<EmulatorCommand>) {

    // --- 1. One-time SDL setup (Unchanged) ---
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let window_canvas = Rc::new(RefCell::new(
        video_subsystem
            .window("JazzNess Emulator", 256 * 2, 240 * 2)
            .position_centered()
            .hidden()
            .build()
            .unwrap()
            .into_canvas()
            .present_vsync()
            .build()
            .unwrap()
    ));

    let texture_creator = window_canvas.borrow().texture_creator();
    let texture = Rc::new(RefCell::new(
        texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
            .unwrap(),
    ));

    let event_pump = Rc::new(RefCell::new(sdl_context.event_pump().unwrap()));

    let desired_spec = AudioSpecDesired {
        freq: Some(AUDIO_SAMPLE_RATE),
        channels: Some(1),
        samples: Some(AUDIO_BUFFER_SIZE),
    };

    let audio_queue = Rc::new(RefCell::new(
        audio_subsystem.open_queue::<f32, _>(None, &desired_spec).unwrap()
    ));
    audio_queue.borrow().resume();

    let mut key_map_init = HashMap::new();
    key_map_init.insert(Keycode::S, joypad::JoypadButton::BUTTON_A);
    key_map_init.insert(Keycode::A, joypad::JoypadButton::BUTTON_B);
    key_map_init.insert(Keycode::Backspace, joypad::JoypadButton::SELECT);
    key_map_init.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map_init.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map_init.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map_init.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map_init.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
    let key_map = Arc::new(key_map_init);

    let rx = Arc::new(Mutex::new(rx));


    // --- 2. "Meta-Loop" (Unchanged) ---
    loop {

        // --- 3. Wait for command (Unchanged) ---
        let command = match rx.lock().unwrap().recv() {
            Ok(cmd) => cmd,
            Err(_) => {
                println!("Emulator Thread: Command channel closed, exiting thread.");
                break;
            }
        };

        let rom_path = match command {
            EmulatorCommand::LoadRom(path) => path,
            EmulatorCommand::SetGameGenieCodes(_) => {
                println!("Emulator Thread: Ignoring cheat codes, no ROM loaded.");
                continue;
            }
            EmulatorCommand::Pause => {
                println!("Emulator Thread: Ignoring pause, no ROM loaded.");
                continue;
            }
        };

        println!("Emulator Thread: Loading ROM: {}", rom_path);
        window_canvas.borrow_mut().window_mut().show();

        // --- 4. Load ROM and set up Bus/CPU (Unchanged) ---
        let mut file = File::open(&rom_path)
            .expect(&format!("Failed to open ROM file: {}", rom_path));
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        let rom = Rom::new(&buffer).unwrap();
        let frame = Rc::new(RefCell::new(Frame::new()));
        let target_frame_time = Duration::from_millis(1000 / 60);

        let window_canvas_clone_loop = Rc::clone(&window_canvas);
        let texture_clone = Rc::clone(&texture);
        let frame_clone = Rc::clone(&frame);
        let audio_queue_clone = Rc::clone(&audio_queue);

        let game_loop = move |ppu: &ppu::NesPPU, _joypad: &mut joypad::Joypad, apu: &mut apu::Apu| {
            let frame_start_time = Instant::now();

            render::render(ppu, &mut frame_clone.borrow_mut());
            texture_clone
                .borrow_mut()
                .update(None, &frame_clone.borrow().data, Frame::WIDTH * 3)
                .unwrap();

            let mut canvas_guard = window_canvas_clone_loop.borrow_mut();
            canvas_guard.copy(&texture_clone.borrow(), None, None).unwrap();
            canvas_guard.present();

            let audio_samples = apu.take_samples();
            if !audio_samples.is_empty() {
                if audio_queue_clone.borrow().size() > (AUDIO_BUFFER_SIZE * 2) as u32 {
                    audio_queue_clone.borrow().clear();
                }
                audio_queue_clone.borrow().queue(&audio_samples);
            }

            let elapsed_time = frame_start_time.elapsed();
            if elapsed_time < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed_time);
            }
        };

        let bus = Bus::new(rom, game_loop);
        
        let paused_flag = bus.debugger.paused.clone();

        let mut cpu = CPU::new(bus);
        cpu.reset();

        // --- 5. Run the inner emulator loop ---
        let instruction_counter = Cell::new(0u32);

        let rx_clone = Arc::clone(&rx);
        let event_pump_clone = Rc::clone(&event_pump);
        // --- FIX 2: Change `Arc::new` to `Arc::clone` ---
        let key_map_clone = Arc::clone(&key_map); 
        let window_canvas_clone_callback = Rc::clone(&window_canvas);

        cpu.run_with_callback(move |cpu| {

            // --- (Rest of file is unchanged) ---

            while paused_flag.load(Ordering::SeqCst) {
                if !handle_debug_prompt(&mut cpu.bus) {
                    println!("Emulator Thread: Quitting from debugger.");
                    window_canvas_clone_callback.borrow_mut().window_mut().hide();
                    std::process::exit(0); 
                }
            }

            match rx_clone.lock().unwrap().try_recv() {
                Ok(EmulatorCommand::LoadRom(_new_path)) => {
                    println!("Emulator Thread: Received new ROM, stopping current emulation.");
                    window_canvas_clone_callback.borrow_mut().window_mut().hide();
                    return false; 
                },
                
                Ok(EmulatorCommand::SetGameGenieCodes(codes)) => {
                    println!("Emulator Thread: Applying Game Genie codes.");
                    cpu.bus.set_game_genie_codes(codes);
                },

                Ok(EmulatorCommand::Pause) => {
                    println!("[DEBUG] Pausing emulator via command.");
                    paused_flag.store(true, Ordering::SeqCst);
                },

                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("Emulator Thread: Menu closed, stopping program.");
                    window_canvas_clone_callback.borrow_mut().window_mut().hide();
                    std::process::exit(0);
                },
                Err(mpsc::TryRecvError::Empty) => { /* No new command */ }
            }

            let count = instruction_counter.get();
            instruction_counter.set(count + 1);
            if count < 1000 { return true; }
            instruction_counter.set(0);

            for event in event_pump_clone.borrow_mut().poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        println!("Emulator Thread: Quit event, hiding window and stopping emulation.");
                        window_canvas_clone_callback.borrow_mut().window_mut().hide();
                        return false; 
                    },
                    Event::KeyDown { keycode, .. } => {
                        if let Some(keycode) = keycode {
                            if let Some(button) = key_map_clone.get(&keycode) {
                                cpu.bus.joypad1.set_button_pressed_status(*button, true);
                            }
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(keycode) = keycode {
                            if let Some(button) = key_map_clone.get(&keycode) {
                                cpu.bus.joypad1.set_button_pressed_status(*button, false);
                            }
                        }
                    }
                    _ => {}
                }
            }

            true 
        });

        audio_queue.borrow().clear();
    }
}


// --- DEBUGGER HELPER FUNCTIONS (Unchanged) ---

/// A helper function to manage the interactive debug prompt.
fn handle_debug_prompt(bus: &mut Bus) -> bool {
    print!("[DEBUG] (c)ontinue, (q)uit, (bp add|rem|list <addr>), (r <addr>), (w <addr> <val>): ");
    io::stdout().flush().unwrap(); 

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        println!("[DEBUG] Error reading input.");
        return true; 
    }

    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    match parts.as_slice() {
        ["c" | "continue"] => {
            println!("[DEBUG] ...resuming");
            bus.debugger.paused.store(false, Ordering::SeqCst);
        }
        ["q" | "quit"] => {
            return false; 
        }
        
        ["bp", "add", addr_str, "r"] => parse_and_add_bp(bus, addr_str, Breakpoint::on_read()),
        ["bp", "add", addr_str, "w"] => parse_and_add_bp(bus, addr_str, Breakpoint::on_write()),
        ["bp", "add", addr_str, "rw"] => parse_and_add_bp(bus, addr_str, Breakpoint::on_rw()),
        ["bp", "add", addr_str] => {
             println!("[DEBUG] Defaulting to Read/Write breakpoint.");
             parse_and_add_bp(bus, addr_str, Breakpoint::on_rw())
        },
        ["bp", "rem", addr_str] => {
            if let Some(addr) = parse_address(addr_str) {
                bus.debugger.remove_breakpoint(addr);
            }
        },
        ["bp", "list"] => {
            println!("[DEBUG] Active Breakpoints:");
            for addr in bus.debugger.get_breakpoints() {
                println!("  - {:#06X}", addr);
            }
        }
        
        ["r" | "read", addr_str] => {
            if let Some(addr) = parse_address(addr_str) {
                let val = bus.mem_read_readonly(addr);
                println!("[DEBUG] Memory at {:#06X} = {:#04X}", addr, val);
            }
        }
        
        ["w" | "write", addr_str, val_str] => {
            if let (Some(addr), Some(val)) = (parse_address(addr_str), parse_value(val_str)) {
                bus.mem_write(addr, val); // This line will now work
                println!("[DEBUG] Wrote {:#04X} to {:#06X}", val, addr);
            }
        }
        
        _ => println!("[DEBUG] Unknown command: '{}'", input.trim()),
    }

    true // Continue
}

/// Helper to parse "0x1234" or "1234" into u16
fn parse_address(addr_str: &str) -> Option<u16> {
    let s = addr_str.trim_start_matches("0x");
    match u16::from_str_radix(s, 16) {
        Ok(addr) => Some(addr),
        Err(e) => {
            println!("[DEBUG] Invalid address '{}': {}", addr_str, e);
            None
        }
    }
}

/// Helper to parse "0x1A" or "1A" into u8
fn parse_value(val_str: &str) -> Option<u8> {
    let s = val_str.trim_start_matches("0x");
    match u8::from_str_radix(s, 16) {
        Ok(val) => Some(val),
        Err(e) => {
            println!("[DEBUG] Invalid value '{}': {}", val_str, e);
            None
        }
    }
}


/// Helper to parse and add a breakpoint
fn parse_and_add_bp(bus: &mut Bus, addr_str: &str, bp: Breakpoint) {
    if let Some(addr) = parse_address(addr_str) {
        bus.debugger.add_breakpoint(addr, bp);
    }
}