pub mod bus;
pub mod cartridge;
pub mod cpu;

use bus::Bus;
use cartridge::Rom;
use cpu::CPU;

use std::env;
use std::fs;

fn main() {
    // --- 1. Load the ROM file ---
    // Expect the path to the NES ROM file as the first command-line argument.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path/to/rom.nes>", args[0]);
        return;
    }
    let rom_path = &args[1];

    // Read the entire ROM file into a vector of bytes.
    let raw_rom = match fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read ROM file '{}': {}", rom_path, e);
            return;
        }
    };

    // --- 2. Initialize Emulator Components ---
    // Parse the raw bytes into a cartridge/ROM structure.
    let rom = match Rom::new(&raw_rom) {
        Ok(rom) => rom,
        Err(e) => {
            eprintln!("Failed to parse ROM: {}", e);
            return;
        }
    };

    // Create the memory bus, passing ownership of the ROM to it.
    let bus = Bus::new(rom);

    // Create the CPU, passing ownership of the bus to it.
    let mut cpu = CPU::new(bus);

    // --- 3. Set Up CPU for the Test ---
    // The nestest.nes ROM requires the program to start at address 0xC000
    // for the automated tests. For a real NES, the CPU would use the reset
    // vector, but for this test, we override it.
    cpu.program_counter = 0xC000;

    // --- 4. Run the Emulator ---
    // Run the CPU with a callback that prints the trace of each instruction.
    // This will continue until the CPU executes a BRK instruction, which is
    // how the test ROM signals it is finished.
    cpu.run_with_callback(move |cpu| {
        println!("{}", cpu.trace());
    });
}