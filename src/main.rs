// --- Keep all module declarations ---
pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod joypad;
pub mod palette;
pub mod ppu;
pub mod render;
pub mod emulator;
pub mod gamegenie;

// --- Add new imports ---
use std::sync::{mpsc, Arc};
use std::thread;

// --- Imports for egui and file dialog ---
use eframe::{self, egui};
use native_dialog::FileDialog;
// --- Import the new command ---
use emulator::EmulatorCommand;


// ##################################################################
// ## 1. `main` function (MODIFIED)
// ## This now spawns the emulator thread and launches the menu.
// ##################################################################

fn main() -> Result<(), eframe::Error> {
    
    // --- 1. Create the communication channel ---
    let (tx, rx) = mpsc::channel::<EmulatorCommand>();

    // --- 2. Spawn the emulator thread ---
    //    It takes ownership of the `rx` (receiver)
    thread::spawn(move || {
        emulator::run_emulator(rx);
    });

    // --- 3. Launch the egui menu on the main thread ---
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("JazzNess - Load ROM")
            .with_inner_size([400.0, 200.0]),
        ..Default::default()
    };

    // We pass the `tx` (transmitter) to the menu app
    eframe::run_native(
        "JazzNess Menu",
        options,
        Box::new(|_cc| Box::new(MenuApp::new(tx))),
    )
}

// ##################################################################
// ## 2. `MenuApp` struct (MODIFIED)
// ## It now holds the `tx` to send commands.
// ##################################################################

struct MenuApp {
    emulator_tx: mpsc::Sender<EmulatorCommand>,
}

impl MenuApp {
    // --- New `new` function to accept the transmitter ---
    fn new(emulator_tx: mpsc::Sender<EmulatorCommand>) -> Self {
        Self { emulator_tx }
    }
}

impl eframe::App for MenuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    
                    // --- "Load ROM..." button (MODIFIED) ---
                    if ui.button("Load ROM...").clicked() {
                        ui.close_menu(); 

                        let path = FileDialog::new()
                            .add_filter("NES ROM", &["nes"])
                            .show_open_single_file()
                            .unwrap_or(None);

                        if let Some(rom_path) = path {
                            // --- Instead of closing, SEND a message ---
                            self.emulator_tx
                                .send(EmulatorCommand::LoadRom(
                                    rom_path.to_string_lossy().to_string()
                                ))
                                .expect("Failed to send LoadRom command");
                        }
                    }

                    // --- "Exit" button (MODIFIED) ---
                    if ui.button("Exit").clicked() {
                        // This will close the egui window. When the `tx`
                        // is dropped, the `rx.recv()` in the emulator
                        // thread will error, and it will shut down.
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // --- Central panel text (unchanged) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Welcome to JazzNess!");
                ui.label("Please use 'File > Load ROM...' to start.");
            });
        });
    }

    // --- `on_exit` is no longer needed ---
}