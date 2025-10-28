#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use native_dialog::FileDialog;
use std::sync::mpsc;
use std::thread;

mod apu;
mod bus;
mod cartridge;
mod cpu;
mod debugger; // --- ADDED: Make sure debugger mod is registered ---
mod emulator;
mod gamegenie;
mod joypad;
mod palette;
mod ppu;
mod render;

use crate::emulator::EmulatorCommand;
use crate::gamegenie::{parse_game_genie_code, GameGenieCode};

struct JazzNessApp {
    emulator_tx: Option<mpsc::Sender<EmulatorCommand>>,
    emulator_thread: Option<thread::JoinHandle<()>>,
    game_genie_codes: Vec<String>,
    cpu_tracing_enabled: bool,
}

impl Default for JazzNessApp {
    fn default() -> Self {
        Self {
            emulator_tx: None,
            emulator_thread: None,
            game_genie_codes: vec!["".to_string(); 6],
            cpu_tracing_enabled: false,
        }
    }
}

impl JazzNessApp {
    fn start_emulator(&mut self, rom_path: String) {
        if let Some(tx) = self.emulator_tx.take() {
            if let Some(handle) = self.emulator_thread.take() {
                if tx.send(EmulatorCommand::LoadRom(rom_path.clone())).is_err() {
                    handle.join().expect("Failed to join emulator thread");
                    self.spawn_new_emulator_thread(rom_path);
                } else {
                    self.emulator_tx = Some(tx);
                }
            }
        } else {
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

    // --- ADDED: Helper to send commands ---
    fn send_command(&self, command: EmulatorCommand) {
        if let Some(tx) = &self.emulator_tx {
            if let Err(e) = tx.send(command) {
                eprintln!("Failed to send command to emulator thread: {}", e);
            }
        } else {
            println!("No emulator running, ignoring command.");
        }
    }
}

impl eframe::App for JazzNessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM...").clicked() {
                        ui.close_menu();
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
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Tools", |ui| {
                    ui.label("Game Genie Codes");
                    ui.separator();

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

                        if !error_messages.is_empty() {
                            native_dialog::MessageDialog::new()
                                .set_type(native_dialog::MessageType::Error)
                                .set_title("Game Genie Error")
                                .set_text(&error_messages.join("\n"))
                                .show_alert()
                                .unwrap();
                        }

                        // --- MODIFIED: Use the helper function ---
                        if !parsed_codes.is_empty() {
                            self.send_command(EmulatorCommand::SetGameGenieCodes(parsed_codes));
                        }

                        if error_messages.is_empty() && self.emulator_tx.is_none() {
                             native_dialog::MessageDialog::new()
                                .set_type(native_dialog::MessageType::Warning)
                                .set_title("Game Genie Warning")
                                .set_text("No ROM is loaded. Cheats cannot be applied.")
                                .show_alert()
                                .unwrap();
                        }

                        ui.close_menu();
                    }
                });
                
                // --- ADDED: A new menu for the debugger ---
                ui.menu_button("Debug", |ui| {
                    // Check if an emulator is running
                    let is_running = self.emulator_tx.is_some();
                    
                    // Add the "Pause" button
                    if ui.add_enabled(is_running, egui::Button::new("Pause")).clicked() {
                        println!("GUI: Sending Pause command.");
                        self.send_command(EmulatorCommand::Pause);
                        ui.close_menu();
                    }

                    // You could add more buttons here later, e.g., to
                    // send breakpoint commands from the GUI

                    ui.separator();
                    if ui.add_enabled(is_running, egui::Checkbox::new(&mut self.cpu_tracing_enabled, "Enable CPU Trace")).changed() {
                        println!("GUI: Setting CPU Tracing to {}", self.cpu_tracing_enabled);
                        self.send_command(EmulatorCommand::SetTracing(self.cpu_tracing_enabled));
                    }
                });
                // --- END ADDED ---
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("JazzNess Emulator");
            ui.separator();
            ui.label("Load a ROM using File > Open ROM...");
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.emulator_tx.take();
        if let Some(handle) = self.emulator_thread.take() {
            handle.join().expect("Failed to join emulator thread");
        }
    }
}

fn main() {
    let options = eframe::NativeOptions {
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