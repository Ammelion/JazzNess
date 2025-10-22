// In src/emulator.rs

// --- Add new imports for Arc and Mutex ---
use std::sync::{mpsc, Arc, Mutex};

// --- Keep all existing imports ---
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::fs::File;
// --- Add Read back ---
use std::io::Read;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::audio::AudioSpecDesired;
// WindowCanvas import no longer needed

use crate::bus::Bus;
use crate::cartridge::Rom;
use crate::cpu::CPU;
use crate::render::frame::Frame;
use crate::render;
use crate::apu;
use crate::ppu;
use crate::joypad;
// --- ADD THIS IMPORT ---
use crate::gamegenie::GameGenieCode;


// --- Audio Constants ---
const AUDIO_SAMPLE_RATE: i32 = 44100;
const AUDIO_BUFFER_SIZE: u16 = 1024;

// --- Emulator Command Enum ---
pub enum EmulatorCommand {
    LoadRom(String),
    // --- ADD THIS VARIANT ---
    SetGameGenieCodes(Vec<GameGenieCode>),
}

pub fn run_emulator(rx: mpsc::Receiver<EmulatorCommand>) {

    // --- 1. One-time SDL setup ---
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    // Store WindowCanvas in Rc<RefCell<>>
    let window_canvas = Rc::new(RefCell::new(
        video_subsystem
            .window("JazzNess Emulator", 256 * 2, 240 * 2)
            .position_centered()
            .hidden() // Start hidden
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

    // Correct AudioSpecDesired initialization
    let desired_spec = AudioSpecDesired {
        freq: Some(AUDIO_SAMPLE_RATE),
        channels: Some(1),
        samples: Some(AUDIO_BUFFER_SIZE),
    };

    // Correct RefCell::new call for audio_queue
    let audio_queue = Rc::new(RefCell::new(
        audio_subsystem.open_queue::<f32, _>(None, &desired_spec).unwrap()
    ));
    audio_queue.borrow().resume();

    // Key Map (wrapped in Arc)
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

    // rx (wrapped in Arc<Mutex<>>)
    let rx = Arc::new(Mutex::new(rx));


    // --- 2. "Meta-Loop" ---
    // Remove unused label 'meta_loop'
    loop {

        // --- 3. Wait for command ---
        let command = match rx.lock().unwrap().recv() {
            Ok(cmd) => cmd,
            Err(_) => {
                println!("Emulator Thread: Command channel closed, exiting thread.");
                break; // Exit the loop cleanly
            }
        };

        // This pattern binding handles the command and avoids unreachable code
        let rom_path = match command {
            EmulatorCommand::LoadRom(path) => path,
            // Handle new command types gracefully if they are received *before* emulation starts
            EmulatorCommand::SetGameGenieCodes(_) => {
                println!("Emulator Thread: Ignoring cheat codes, no ROM loaded.");
                continue; // Go back to waiting for a LoadRom command
            }
        };

        println!("Emulator Thread: Loading ROM: {}", rom_path);

        // --- FIX: Use borrow_mut().window_mut().show() ---
        window_canvas.borrow_mut().window_mut().show();

        // --- 4. Load ROM and set up Bus/CPU ---
        // Add buffer creation and file reading
        let mut file = File::open(&rom_path)
            .expect(&format!("Failed to open ROM file: {}", rom_path));
        let mut buffer = Vec::new(); // <-- Was missing
        file.read_to_end(&mut buffer).unwrap(); // <-- Was missing

        let rom = Rom::new(&buffer).unwrap();
        let frame = Rc::new(RefCell::new(Frame::new()));
        let target_frame_time = Duration::from_millis(1000 / 60);

        // --- Create clones for the `game_loop` closure ---
        let window_canvas_clone_loop = Rc::clone(&window_canvas);
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

            let mut canvas_guard = window_canvas_clone_loop.borrow_mut();
            canvas_guard.copy(&texture_clone.borrow(), None, None).unwrap();
            canvas_guard.present();

            // Queue audio
            let audio_samples = apu.take_samples();
            if !audio_samples.is_empty() {
                if audio_queue_clone.borrow().size() > (AUDIO_BUFFER_SIZE * 2) as u32 {
                    audio_queue_clone.borrow().clear();
                }
                audio_queue_clone.borrow().queue(&audio_samples);
            }

            // Sleep
            let elapsed_time = frame_start_time.elapsed();
            if elapsed_time < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed_time);
            }
        };

        let bus = Bus::new(rom, game_loop);
        let mut cpu = CPU::new(bus);
        cpu.reset();

        // --- 5. Run the inner emulator loop ---
        let instruction_counter = Cell::new(0u32);

        // --- Create clones before the `run_with_callback` closure ---
        let rx_clone = Arc::clone(&rx);
        let event_pump_clone = Rc::clone(&event_pump);
        let key_map_clone = Arc::clone(&key_map);
        let window_canvas_clone_callback = Rc::clone(&window_canvas);


        cpu.run_with_callback(move |cpu| {

            // --- Check for new commands (non-blocking) ---
            match rx_clone.lock().unwrap().try_recv() {
                Ok(EmulatorCommand::LoadRom(_new_path)) => {
                    println!("Emulator Thread: Received new ROM, stopping current emulation.");
                    // --- FIX: Use borrow_mut().window_mut().hide() ---
                    window_canvas_clone_callback.borrow_mut().window_mut().hide();
                    return false; // Stop CPU loop
                },
                
                // --- ADD THIS HANDLER ---
                Ok(EmulatorCommand::SetGameGenieCodes(codes)) => {
                    println!("Emulator Thread: Applying Game Genie codes.");
                    // Pass the new codes to the bus
                    cpu.bus.set_game_genie_codes(codes);
                    // Don't return, just continue emulation
                },
                // --- END OF NEW HANDLER ---

                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("Emulator Thread: Menu closed, stopping program.");
                    // --- FIX: Use borrow_mut().window_mut().hide() ---
                    window_canvas_clone_callback.borrow_mut().window_mut().hide();
                    std::process::exit(0); // Exit whole app if menu closes
                },
                Err(mpsc::TryRecvError::Empty) => { /* No new command */ }
            }

            // --- Throttling logic ---
            let count = instruction_counter.get();
            instruction_counter.set(count + 1);
            if count < 1000 { return true; }
            instruction_counter.set(0);

            // --- Handle SDL events ---
            for event in event_pump_clone.borrow_mut().poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        println!("Emulator Thread: Quit event, hiding window and stopping emulation.");
                        // --- FIX: Use borrow_mut().window_mut().hide() ---
                        window_canvas_clone_callback.borrow_mut().window_mut().hide();
                        return false; // Stop CPU loop
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

            true // --- Continue running ---
        });

        // --- Loop Cleanup ---
        audio_queue.borrow().clear();
    }
}