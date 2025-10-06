mod bus;
mod cartridge;
mod cpu;

use bus::{Bus, Mem};
use cartridge::Rom;
use cpu::CPU;
use std::fs;

fn main() {
    // Load the test ROM
    let raw_rom = fs::read("nestest.nes").expect("Failed to read nestest.nes");
    let rom = Rom::new(&raw_rom).expect("Failed to parse nestest.nes");
    let bus = Bus::new(rom);
    let mut cpu = CPU::new(bus);

    // Set the program counter to 0xC000, which is the starting point for this test ROM
    cpu.program_counter = 0xC000;

    // Run the CPU with a callback that prints the trace of each instruction
    cpu.run_with_callback(move |cpu| {
        println!("{}", cpu.trace());
    });
}