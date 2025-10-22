#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use native_dialog::FileDialog;
use std::sync::mpsc;
use std::thread;

mod apu;
mod bus;
mod cartridge;
mod cpu;
mod emulator;
mod gamegenie; // Make gamegenie module visible
mod joypad;
mod palette;
mod ppu;
mod render;

// --- ADD THESE IMPORTS ---
use crate::emulator::EmulatorCommand;
use crate::gamegenie::{parse_game_genie_code, GameGenieCode};
// --- END IMPORTS ---

struct JazzNessApp {
    // Sender for emulator commands
    emulator_tx: Option<mpsc::Sender<EmulatorCommand>>,
    // Handle to the emulator thread
    emulator_thread: Option<thread::JoinHandle<()>>,

    // --- ADD STATE FOR GAME GENIE CODES ---
    game_genie_codes: Vec<String>,
}

impl Default for JazzNessApp {
    fn default() -> Self {
        Self {
            emulator_tx: None,
            emulator_thread: None,
            // --- INITIALIZE GAME GENIE STATE (6 slots) ---
            game_genie_codes: vec!["".to_string(); 6],
        }
    }
}

impl JazzNessApp {
    fn start_emulator(&mut self, rom_path: String) {
        // If an emulator is already running, stop it
        if let Some(tx) = self.emulator_tx.take() {
            // Send a command to load the new ROM. The emulator thread will handle this.
            // (Or you could design a "Stop" command)
            // For now, we'll just let the old thread die when we drop the new sender
            // A cleaner way would be to send a Stop command and join the thread.
            // But for this structure, we'll just start a new one.
            if let Some(handle) = self.emulator_thread.take() {
                // This is a bit simplified. A real implementation
                // would need a "Stop" command and then join.
                // For now, we'll just orphan the old thread.
                // Let's try to send a new ROM load, which will restart its inner loop.
                if tx.send(EmulatorCommand::LoadRom(rom_path.clone())).is_err() {
                    // Thread probably died, join it.
                    handle.join().expect("Failed to join emulator thread");
                    // And start a new one
                    self.spawn_new_emulator_thread(rom_path);
                } else {
                    // Emulator is still alive, reuse it
                    self.emulator_tx = Some(tx);
                }
            }
        } else {
            // No emulator running, start a new one
            self.spawn_new_emulator_thread(rom_path);
        }
    }

    fn spawn_new_emulator_thread(&mut self, rom_path: String) {
        let (tx, rx) = mpsc::channel();
        let emulator_handle = thread::spawn(move || {
            emulator::run_emulator(rx);
        });

        tx.send(EmulatorCommand::LoadRom(rom_path))
            .expect("Failed to send initial ROM load command");

        self.emulator_tx = Some(tx);
        self.emulator_thread = Some(emulator_handle);
    }
}

impl eframe::App for JazzNessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM...").clicked() {
                        ui.close_menu(); // Close the menu before opening the dialog
                        let result = FileDialog::new()
                            .set_location("~")
                            .add_filter("NES ROM", &["nes"])
                            .show_open_single_file();

                        match result {
                            Ok(Some(path)) => {
                                if let Some(path_str) = path.to_str() {
                                    self.start_emulator(path_str.to_string());
                                }
                            }
                            Ok(None) => { /* User cancelled */ }
                            Err(e) => {
                                // Show error dialog
                                native_dialog::MessageDialog::new()
                                    .set_type(native_dialog::MessageType::Error)
                                    .set_title("Error Opening File")
                                    .set_text(&e.to_string())
                                    .show_alert()
                                    .unwrap();
                            }
                        }
                    }
                    if ui.button("Exit").clicked() {
                        // --- FIX 1: Use ctx.send_viewport_cmd to close ---
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // --- ADD THE "TOOLS" MENU ---
                ui.menu_button("Tools", |ui| {
                    ui.label("Game Genie Codes");
                    ui.separator();

                    // Add text boxes for the codes
                    for code_str in self.game_genie_codes.iter_mut() {
                        ui.add(
                            egui::TextEdit::singleline(code_str)
                                .hint_text("AAPPZK")
                                .desired_width(100.0),
                        );
                    }

                    ui.separator();

                    if ui.button("Apply Cheats").clicked() {
                        let mut parsed_codes = Vec::<GameGenieCode>::new();
                        let mut error_messages = Vec::<String>::new();

                        // Parse all codes
                        for (i, code_str) in self.game_genie_codes.iter().enumerate() {
                            if !code_str.is_empty() {
                                match parse_game_genie_code(code_str) {
                                    Ok(code) => parsed_codes.push(code),
                                    Err(e) => {
                                        let msg = format!(
                                            "Slot {}: Failed to parse '{}' - {}",
                                            i + 1,
                                            code_str,
                                            e
                                        );
                                        error_messages.push(msg);
                                    }
                                }
                            }
                        }

                        // Show errors if any
                        if !error_messages.is_empty() {
                            native_dialog::MessageDialog::new()
                                .set_type(native_dialog::MessageType::Error)
                                .set_title("Game Genie Error")
                                .set_text(&error_messages.join("\n"))
                                .show_alert()
                                .unwrap();
                        }

                        // Send valid codes to the emulator thread
                        if let Some(tx) = &self.emulator_tx {
                            if let Err(e) = tx.send(EmulatorCommand::SetGameGenieCodes(parsed_codes))
                            {
                                eprintln!("Failed to send cheat codes to emulator thread: {}", e);
                                native_dialog::MessageDialog::new()
                                    .set_type(native_dialog::MessageType::Error)
                                    .set_title("Emulator Error")
                                    .set_text(
                                        "Failed to send cheat codes. Emulator thread may have crashed.",
                                    )
                                    .show_alert()
                                    .unwrap();
                            }
                        } else {
                            // Only show this warning if there were no parsing errors
                            if error_messages.is_empty() {
                                native_dialog::MessageDialog::new()
                                    .set_type(native_dialog::MessageType::Warning)
                                    .set_title("Game Genie Warning")
                                    .set_text("No ROM is loaded. Cheats cannot be applied.")
                                    .show_alert()
                                    .unwrap();
                            }
                        }

                        ui.close_menu();
                    }
                });
                // --- END OF "TOOLS" MENU ---
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // This is where the game *would* be rendered if we were
            // rendering directly into the egui context.
            // Since rendering happens in the emulator thread's SDL window,
            // this central panel will just be a blank background.
            ui.label("JazzNess Emulator");
            ui.separator();
            ui.label("Load a ROM using File > Open ROM...");
        });
    }

    // --- FIX 2: Change signature to match trait ---
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // On exit, we should gracefully shut down the emulator thread.
        // Dropping the sender will cause the receiver in the emulator thread
        // to get an error, which will stop its loop.
        self.emulator_tx.take();
        if let Some(handle) = self.emulator_thread.take() {
            handle.join().expect("Failed to join emulator thread");
        }
    }
}

fn main() {
    let options = eframe::NativeOptions {
        // --- FIX 3: Use viewport builder for initial size ---
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native(
        "JazzNess",
        options,
        Box::new(|_cc| Box::<JazzNessApp>::default()),
    )
    .expect("Failed to run eframe");
}