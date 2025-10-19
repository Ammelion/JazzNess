use crate::bus::{Bus, Mem};
use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
    Indirect_X,
    Indirect_Y,
    Relative,
    Implied,
    Accumulator,
}

//Status Flag Constants
const CARRY_FLAG: u8 = 0b0000_0001;
const ZERO_FLAG: u8 = 0b0000_0010;
const INTERRUPT_DISABLE: u8 = 0b0000_0100;
const DECIMAL_MODE: u8 = 0b0000_1000; // Not used by NES
const BREAK_COMMAND: u8 = 0b0001_0000;
const BREAK_COMMAND_2: u8 = 0b0010_0000;
const OVERFLOW_FLAG: u8 = 0b0100_0000;
const NEGATIVE_FLAG: u8 = 0b1000_0000;

pub struct CPU<'call> {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub stack_pointer: u8,
    pub status: u8,
    pub program_counter: u16,
    bus: Bus<'call>,
}
pub struct OpCode {
    pub code: u8,
    pub name: &'static str,
    pub bytes: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

// NOTE: The `impl Mem for CPU` block is now removed as it's no longer needed.

impl OpCode {
    fn new(code: u8, name: &'static str, bytes: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode {
            code,
            name,
            bytes,
            cycles,
            mode,
        }
    }
}

lazy_static! {
    pub static ref CPU_OPCODES: Vec<OpCode> = vec![
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::Implied),
        OpCode::new(0xEA, "NOP", 1, 2, AddressingMode::Implied),
        OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7D, "ADC", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x79, "ADC", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x71, "ADC", 2, 5, AddressingMode::Indirect_Y),
        OpCode::new(0xE9, "SBC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE5, "SBC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xF5, "SBC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xED, "SBC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xFD, "SBC", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xF9, "SBC", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xE1, "SBC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xF1, "SBC", 2, 5, AddressingMode::Indirect_Y),
        OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3D, "AND", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x39, "AND", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x31, "AND", 2, 5, AddressingMode::Indirect_Y),
        OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x4D, "EOR", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5D, "EOR", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x59, "EOR", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x41, "EOR", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x51, "EOR", 2, 5, AddressingMode::Indirect_Y),
        OpCode::new(0x09, "ORA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0D, "ORA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1D, "ORA", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x19, "ORA", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x01, "ORA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x11, "ORA", 2, 5, AddressingMode::Indirect_Y),
        OpCode::new(0x0A, "ASL", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1E, "ASL", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x4A, "LSR", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4E, "LSR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5E, "LSR", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x2A, "ROL", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2E, "ROL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3E, "ROL", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x6A, "ROR", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6E, "ROR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7E, "ROR", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xB1, "LDA", 2, 5, AddressingMode::Indirect_Y),
        OpCode::new(0xA2, "LDX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA6, "LDX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB6, "LDX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xAE, "LDX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBE, "LDX", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xA0, "LDY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA4, "LDY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB4, "LDY", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAC, "LDY", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBC, "LDY", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8D, "STA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9D, "STA", 3, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),
        OpCode::new(0x86, "STX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x96, "STX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x8E, "STX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x84, "STY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x94, "STY", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8C, "STY", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x18, "CLC", 1, 2, AddressingMode::Implied),
        OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::Implied),
        OpCode::new(0x58, "CLI", 1, 2, AddressingMode::Implied),
        OpCode::new(0xB8, "CLV", 1, 2, AddressingMode::Implied),
        OpCode::new(0x38, "SEC", 1, 2, AddressingMode::Implied),
        OpCode::new(0xF8, "SED", 1, 2, AddressingMode::Implied),
        OpCode::new(0x78, "SEI", 1, 2, AddressingMode::Implied),
        OpCode::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xDD, "CMP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xD9, "CMP", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xC1, "CMP", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xD1, "CMP", 2, 5, AddressingMode::Indirect_Y),
        OpCode::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xCC, "CPY", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x10, "BPL", 2, 2, AddressingMode::Relative),
        OpCode::new(0x30, "BMI", 2, 2, AddressingMode::Relative),
        OpCode::new(0x50, "BVC", 2, 2, AddressingMode::Relative),
        OpCode::new(0x70, "BVS", 2, 2, AddressingMode::Relative),
        OpCode::new(0x90, "BCC", 2, 2, AddressingMode::Relative),
        OpCode::new(0xB0, "BCS", 2, 2, AddressingMode::Relative),
        OpCode::new(0xD0, "BNE", 2, 2, AddressingMode::Relative),
        OpCode::new(0xF0, "BEQ", 2, 2, AddressingMode::Relative),
        OpCode::new(0x4C, "JMP", 3, 3, AddressingMode::Absolute),
        OpCode::new(0x6C, "JMP", 3, 5, AddressingMode::Indirect),
        OpCode::new(0x20, "JSR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x60, "RTS", 1, 6, AddressingMode::Implied),
        OpCode::new(0x40, "RTI", 1, 6, AddressingMode::Implied),
        OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xC6, "DEC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xD6, "DEC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xCE, "DEC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xDE, "DEC", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xCA, "DEX", 1, 2, AddressingMode::Implied),
        OpCode::new(0x88, "DEY", 1, 2, AddressingMode::Implied),
        OpCode::new(0xE6, "INC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xF6, "INC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xEE, "INC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xFE, "INC", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xE8, "INX", 1, 2, AddressingMode::Implied),
        OpCode::new(0xC8, "INY", 1, 2, AddressingMode::Implied),
        OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::Implied),
        OpCode::new(0xA8, "TAY", 1, 2, AddressingMode::Implied),
        OpCode::new(0xBA, "TSX", 1, 2, AddressingMode::Implied),
        OpCode::new(0x8A, "TXA", 1, 2, AddressingMode::Implied),
        OpCode::new(0x9A, "TXS", 1, 2, AddressingMode::Implied),
        OpCode::new(0x98, "TYA", 1, 2, AddressingMode::Implied),
        OpCode::new(0x48, "PHA", 1, 3, AddressingMode::Implied),
        OpCode::new(0x08, "PHP", 1, 3, AddressingMode::Implied),
        OpCode::new(0x68, "PLA", 1, 4, AddressingMode::Implied),
        OpCode::new(0x28, "PLP", 1, 4, AddressingMode::Implied),

        // Unofficial Opcodes follow...
        OpCode::new(0x1A, "*NOP", 1, 2, AddressingMode::Implied),
        OpCode::new(0x3A, "*NOP", 1, 2, AddressingMode::Implied),
        OpCode::new(0x5A, "*NOP", 1, 2, AddressingMode::Implied),
        OpCode::new(0x7A, "*NOP", 1, 2, AddressingMode::Implied),
        OpCode::new(0xDA, "*NOP", 1, 2, AddressingMode::Implied),
        OpCode::new(0xFA, "*NOP", 1, 2, AddressingMode::Implied),
        OpCode::new(0x80, "*NOP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x82, "*NOP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x89, "*NOP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC2, "*NOP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE2, "*NOP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x04, "*NOP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x44, "*NOP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x64, "*NOP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x14, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x34, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x54, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x74, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xD4, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xF4, "*NOP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0C, "*NOP", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1C, "*NOP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x3C, "*NOP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x5C, "*NOP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x7C, "*NOP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xDC, "*NOP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xFC, "*NOP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x0B, "*AAC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x2B, "*AAC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x87, "*SAX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x97, "*SAX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x83, "*SAX", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x8F, "*SAX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x6B, "*ARR", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x4B, "*ASR", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xAB, "*ATX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x9F, "*AXA", 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x93, "*AXA", 2, 6, AddressingMode::Indirect_Y),
        OpCode::new(0xCB, "*AXS", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC7, "*DCP", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xD7, "*DCP", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xCF, "*DCP", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xDF, "*DCP", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xDB, "*DCP", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0xC3, "*DCP", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0xD3, "*DCP", 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0xE7, "*ISB", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xF7, "*ISB", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xEF, "*ISB", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xFF, "*ISB", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xFB, "*ISB", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0xE3, "*ISB", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0xF3, "*ISB", 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0x02, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0x12, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0x22, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0x32, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0x42, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0x52, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0x62, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0x72, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0x92, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0xB2, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0xD2, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0xF2, "*KIL", 1, 0, AddressingMode::Implied),
        OpCode::new(0xBB, "*LAR", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xA7, "*LAX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB7, "*LAX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xAF, "*LAX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBF, "*LAX", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xA3, "*LAX", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xB3, "*LAX", 2, 5, AddressingMode::Indirect_Y),
        OpCode::new(0x27, "*RLA", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x37, "*RLA", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2F, "*RLA", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3F, "*RLA", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x3B, "*RLA", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x23, "*RLA", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x33, "*RLA", 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0x67, "*RRA", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x77, "*RRA", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6F, "*RRA", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7F, "*RRA", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x7B, "*RRA", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x63, "*RRA", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x73, "*RRA", 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0xEB, "*SBC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x07, "*SLO", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x17, "*SLO", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0F, "*SLO", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1F, "*SLO", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x1B, "*SLO", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x03, "*SLO", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x13, "*SLO", 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0x47, "*SRE", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x57, "*SRE", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4F, "*SRE", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5F, "*SRE", 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x5B, "*SRE", 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x43, "*SRE", 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x53, "*SRE", 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0x9E, "*SXA", 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x9C, "*SYA", 3, 5, AddressingMode::Absolute_X),
        OpCode::new(0x8B, "*XAA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x9B, "*XAS", 3, 5, AddressingMode::Absolute_Y),
    ];
}

impl<'call> CPU<'call> {
    // CHANGE: Constructor now takes a Bus
    pub fn new(bus: Bus<'call>) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stack_pointer: 0xFD,
            status: INTERRUPT_DISABLE | BREAK_COMMAND_2,
            program_counter: 0,
            bus,
        }
    }
    // --- Private Helper Methods now use the Bus ---
    fn stack_push(&mut self, data: u8) {
        self.bus.mem_write(0x0100 + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn stack_pull(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.bus.mem_read(0x0100 + self.stack_pointer as u16)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00FF) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pull_u16(&mut self) -> u16 {
        let lo = self.stack_pull() as u16;
        let hi = self.stack_pull() as u16;
        (hi << 8) | lo
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter + 1,

            AddressingMode::ZeroPage => self.bus.mem_read(self.program_counter + 1) as u16,

            AddressingMode::Absolute => self.bus.mem_read_u16(self.program_counter + 1),

            AddressingMode::ZeroPage_X => {
                let pos = self.bus.mem_read(self.program_counter + 1);
                pos.wrapping_add(self.register_x) as u16
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.bus.mem_read(self.program_counter + 1);
                pos.wrapping_add(self.register_y) as u16
            }

            AddressingMode::Absolute_X => {
                let base = self.bus.mem_read_u16(self.program_counter + 1);
                base.wrapping_add(self.register_x as u16)
            }

            AddressingMode::Absolute_Y => {
                let base = self.bus.mem_read_u16(self.program_counter + 1);
                base.wrapping_add(self.register_y as u16)
            }

            AddressingMode::Indirect => {
                let ptr = self.bus.mem_read_u16(self.program_counter + 1);
                // Emulate 6502 bug
                if ptr & 0x00FF == 0x00FF {
                    let lo = self.bus.mem_read(ptr);
                    let hi = self.bus.mem_read(ptr & 0xFF00);
                    (hi as u16) << 8 | (lo as u16)
                } else {
                    self.bus.mem_read_u16(ptr)
                }
            }

            AddressingMode::Indirect_X => {
                let base = self.bus.mem_read(self.program_counter + 1);
                let ptr: u8 = base.wrapping_add(self.register_x);
                let lo = self.bus.mem_read(ptr as u16);
                let hi = self.bus.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::Indirect_Y => {
                let base = self.bus.mem_read(self.program_counter + 1);
                let lo = self.bus.mem_read(base as u16);
                let hi = self.bus.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                deref_base.wrapping_add(self.register_y as u16)
            }
            
            AddressingMode::Relative => {
                let offset = self.bus.mem_read(self.program_counter + 1) as i8;
                self.program_counter.wrapping_add(2).wrapping_add(offset as u16)
            }

            AddressingMode::Implied | AddressingMode::Accumulator => {
                panic!("mode {:?} is not supported for memory addressing", mode);
            }
        }
    }

    fn get_operand(&mut self, mode: &AddressingMode) -> u8 {
    match mode {
            AddressingMode::Accumulator => self.register_a,
            _ => {
                let addr = self.get_operand_address(mode);
                self.bus.mem_read(addr)
            }
        }
    }

    fn set_operand(&mut self, mode: &AddressingMode, value: u8) {
        match mode {
            AddressingMode::Accumulator => self.register_a = value,
            _ => {
                let addr = self.get_operand_address(mode);
                self.bus.mem_write(addr, value);
            }
        }
    }

    fn set_flag(&mut self, flag: u8, val: bool) {
        if val {
            self.status |= flag;
        } else {
            self.status &= !flag;
        }
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.status & flag) > 0
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.set_flag(ZERO_FLAG, result == 0);
        self.set_flag(NEGATIVE_FLAG, (result & 0b1000_0000) != 0);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = 0xFD;
        self.status = INTERRUPT_DISABLE | BREAK_COMMAND_2;
        self.program_counter = self.bus.mem_read_u16(0xFFFC);
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            self.program_counter = self.get_operand_address(&AddressingMode::Relative);
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        let sum = self.register_a as u16 + value as u16 + self.get_flag(CARRY_FLAG) as u16;
        self.set_flag(CARRY_FLAG, sum > 0xFF);
        let result = sum as u8;
        self.set_flag(
            OVERFLOW_FLAG,
            (self.register_a ^ result) & (value ^ result) & 0x80 != 0,
        );
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        let addend = !value;
        let sum = self.register_a as u16 + addend as u16 + self.get_flag(CARRY_FLAG) as u16;
        self.set_flag(CARRY_FLAG, sum > 0xFF);
        let result = sum as u8;
        self.set_flag(
            OVERFLOW_FLAG,
            (self.register_a ^ result) & (addend ^ result) & 0x80 != 0,
        );
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn compare(&mut self, mode: &AddressingMode, register: u8) {
        let value = self.get_operand(mode);
        self.set_flag(CARRY_FLAG, register >= value);
        self.update_zero_and_negative_flags(register.wrapping_sub(value));
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static OpCode> =
            CPU_OPCODES.iter().map(|op| (op.code, op)).collect();

        loop {
            if self.bus.poll_nmi_status().is_some() {
                self.interrupt_nmi();
            }
            //println!("{}", self.trace());
            callback(self);
            
            let code = self.bus.mem_read(self.program_counter);
            let opcode_ref = opcodes
                .get(&code)
                .unwrap_or_else(|| panic!("OpCode {:x} is not recognized", code));

            let pc_state = self.program_counter;

            let mode = &opcode_ref.mode;
            let name = opcode_ref.name;
            
            match name {
                "BRK" => {
                    self.stack_push_u16(self.program_counter + 1);
                    let mut status = self.status;
                    status |= BREAK_COMMAND; // Set B flag to 1
                    status |= BREAK_COMMAND_2;
                    self.stack_push(status);

                    self.set_flag(INTERRUPT_DISABLE, true);
                    self.program_counter = self.bus.mem_read_u16(0xFFFE);
                }
                "NOP" => {}

                /* Load/Store */
                "LDA" => {
                    self.register_a = self.get_operand(mode);
                    self.update_zero_and_negative_flags(self.register_a);
                }
                "LDX" => {
                    self.register_x = self.get_operand(mode);
                    self.update_zero_and_negative_flags(self.register_x);
                }
                "LDY" => {
                    self.register_y = self.get_operand(mode);
                    self.update_zero_and_negative_flags(self.register_y);
                }
                "STA" => {
                    self.set_operand(mode, self.register_a);
                }
                "STX" => {
                    self.set_operand(mode, self.register_x);
                }
                "STY" => {
                    self.set_operand(mode, self.register_y);
                }

                /* Arithmetic */
                "ADC" => self.adc(mode),
                "SBC" => self.sbc(mode),
                "AND" => {
                    self.register_a &= self.get_operand(mode);
                    self.update_zero_and_negative_flags(self.register_a);
                }
                "EOR" => {
                    self.register_a ^= self.get_operand(mode);
                    self.update_zero_and_negative_flags(self.register_a);
                }
                "ORA" => {
                    self.register_a |= self.get_operand(mode);
                    self.update_zero_and_negative_flags(self.register_a);
                }

                /* Shifts */
                "ASL" => {
                    let mut val = self.get_operand(mode);
                    self.set_flag(CARRY_FLAG, val & 0x80 != 0);
                    val <<= 1;
                    self.set_operand(mode, val);
                    self.update_zero_and_negative_flags(val);
                }
                "LSR" => {
                    let mut val = self.get_operand(mode);
                    self.set_flag(CARRY_FLAG, val & 0x01 != 0);
                    val >>= 1;
                    self.set_operand(mode, val);
                    self.update_zero_and_negative_flags(val);
                }
                "ROL" => {
                    let mut val = self.get_operand(mode);
                    let c = self.get_flag(CARRY_FLAG);
                    self.set_flag(CARRY_FLAG, val & 0x80 != 0);
                    val <<= 1;
                    if c {
                        val |= 1;
                    };
                    self.set_operand(mode, val);
                    self.update_zero_and_negative_flags(val);
                }
                "ROR" => {
                    let mut val = self.get_operand(mode);
                    let c = self.get_flag(CARRY_FLAG);
                    self.set_flag(CARRY_FLAG, val & 0x01 != 0);
                    val >>= 1;
                    if c {
                        val |= 0x80;
                    };
                    self.set_operand(mode, val);
                    self.update_zero_and_negative_flags(val);
                }

                /* INC/DEC */
                "INC" => {
                    let mut val = self.get_operand(mode);
                    val = val.wrapping_add(1);
                    self.set_operand(mode, val);
                    self.update_zero_and_negative_flags(val);
                }
                "INX" => {
                    self.register_x = self.register_x.wrapping_add(1);
                    self.update_zero_and_negative_flags(self.register_x);
                }
                "INY" => {
                    self.register_y = self.register_y.wrapping_add(1);
                    self.update_zero_and_negative_flags(self.register_y);
                }
                "DEC" => {
                    let mut val = self.get_operand(mode);
                    val = val.wrapping_sub(1);
                    self.set_operand(mode, val);
                    self.update_zero_and_negative_flags(val);
                }
                "DEX" => {
                    self.register_x = self.register_x.wrapping_sub(1);
                    self.update_zero_and_negative_flags(self.register_x);
                }
                "DEY" => {
                    self.register_y = self.register_y.wrapping_sub(1);
                    self.update_zero_and_negative_flags(self.register_y);
                }

                /* Compare */
                "CMP" => self.compare(mode, self.register_a),
                "CPX" => self.compare(mode, self.register_x),
                "CPY" => self.compare(mode, self.register_y),

                /* Jumps */
                "JMP" => self.program_counter = self.get_operand_address(mode),
                "JSR" => {
                    self.stack_push_u16(self.program_counter + 2);
                    self.program_counter = self.get_operand_address(mode);
                }
                "RTS" => self.program_counter = self.stack_pull_u16().wrapping_add(1),
                "RTI" => {
                    self.status = self.stack_pull();
                    self.set_flag(BREAK_COMMAND, false);
                    self.set_flag(BREAK_COMMAND_2, true);
                    self.program_counter = self.stack_pull_u16();
                }

                /* Branches */
                "BCC" => self.branch(!self.get_flag(CARRY_FLAG)),
                "BCS" => self.branch(self.get_flag(CARRY_FLAG)),
                "BEQ" => self.branch(self.get_flag(ZERO_FLAG)),
                "BNE" => self.branch(!self.get_flag(ZERO_FLAG)),
                "BMI" => self.branch(self.get_flag(NEGATIVE_FLAG)),
                "BPL" => self.branch(!self.get_flag(NEGATIVE_FLAG)),
                "BVC" => self.branch(!self.get_flag(OVERFLOW_FLAG)),
                "BVS" => self.branch(self.get_flag(OVERFLOW_FLAG)),

                /* Flags */
                "CLC" => self.set_flag(CARRY_FLAG, false),
                "CLD" => self.set_flag(DECIMAL_MODE, false),
                "CLI" => self.set_flag(INTERRUPT_DISABLE, false),
                "CLV" => self.set_flag(OVERFLOW_FLAG, false),
                "SEC" => self.set_flag(CARRY_FLAG, true),
                "SED" => self.set_flag(DECIMAL_MODE, true),
                "SEI" => self.set_flag(INTERRUPT_DISABLE, true),

                /* Stack */
                "PHA" => self.stack_push(self.register_a),
                "PHP" => {
                    self.stack_push(self.status | BREAK_COMMAND | BREAK_COMMAND_2);
                }
                "PLA" => {
                    self.register_a = self.stack_pull();
                    self.update_zero_and_negative_flags(self.register_a);
                }
                "PLP" => {
                    self.status = self.stack_pull();
                    self.set_flag(BREAK_COMMAND, false);
                    self.set_flag(BREAK_COMMAND_2, true);
                }

                /* Transfers */
                "TAX" => {
                    self.register_x = self.register_a;
                    self.update_zero_and_negative_flags(self.register_x);
                }
                "TAY" => {
                    self.register_y = self.register_a;
                    self.update_zero_and_negative_flags(self.register_y);
                }
                "TSX" => {
                    self.register_x = self.stack_pointer;
                    self.update_zero_and_negative_flags(self.register_x);
                }
                "TXA" => {
                    self.register_a = self.register_x;
                    self.update_zero_and_negative_flags(self.register_a);
                }
                "TXS" => self.stack_pointer = self.register_x,
                "TYA" => {
                    self.register_a = self.register_y;
                    self.update_zero_and_negative_flags(self.register_a);
                }

                /* Other */
                "BIT" => {
                    let val = self.get_operand(mode);
                    self.set_flag(ZERO_FLAG, (self.register_a & val) == 0);
                    self.set_flag(NEGATIVE_FLAG, val & NEGATIVE_FLAG != 0);
                    self.set_flag(OVERFLOW_FLAG, val & OVERFLOW_FLAG != 0);
                }
                "*NOP" => { }

                "*KIL" => { panic!("KIL instruction executed."); }

                "*SBC" => {
                    self.sbc(mode);
                }

                "*AAC" => {
                    let value = self.get_operand(mode);
                    self.register_a &= value;
                    self.update_zero_and_negative_flags(self.register_a);
                    if self.get_flag(NEGATIVE_FLAG) {
                        self.set_flag(CARRY_FLAG, true);
                    }
                }
                
                "*SAX" => {
                    let value = self.register_a & self.register_x;
                    self.set_operand(mode, value);
                }

                "*ARR" => {
                    let value = self.get_operand(mode);
                    self.register_a &= value;
                    self.register_a = (self.register_a >> 1) | (if self.get_flag(CARRY_FLAG) { 0x80 } else { 0 });
                    self.update_zero_and_negative_flags(self.register_a);

                    let bit6 = (self.register_a & 0b0100_0000) != 0;
                    let bit5 = (self.register_a & 0b0010_0000) != 0;

                    match (bit6, bit5) {
                        (true, true)   => { self.set_flag(CARRY_FLAG, true); self.set_flag(OVERFLOW_FLAG, false); },
                        (false, false) => { self.set_flag(CARRY_FLAG, false); self.set_flag(OVERFLOW_FLAG, false); },
                        (false, true)  => { self.set_flag(CARRY_FLAG, false); self.set_flag(OVERFLOW_FLAG, true); },
                        (true, false)  => { self.set_flag(CARRY_FLAG, true); self.set_flag(OVERFLOW_FLAG, true); },
                    }
                }

                "*ASR" => {
                    let value = self.get_operand(mode);
                    self.register_a &= value;
                    self.set_flag(CARRY_FLAG, (self.register_a & 0x01) != 0);
                    self.register_a >>= 1;
                    self.update_zero_and_negative_flags(self.register_a);
                }

                "*ATX" => {
                    let value = self.get_operand(mode);
                    self.register_a &= value;
                    self.register_x = self.register_a;
                    self.update_zero_and_negative_flags(self.register_x);
                }
                
                "*AXA" => {
                    let addr = self.get_operand_address(mode);
                    let value = self.register_a & self.register_x & 7;
                    self.bus.mem_write(addr, value);
                }

                "*AXS" => {
                    let value = self.get_operand(mode);
                    let start_val = self.register_a & self.register_x;
                    let (result, borrow) = start_val.overflowing_sub(value);
                    self.register_x = result;
                    self.set_flag(CARRY_FLAG, !borrow);
                    self.update_zero_and_negative_flags(self.register_x);
                }

                "*DCP" => {
                    let addr = self.get_operand_address(mode);
                    let mut value = self.bus.mem_read(addr);
                    value = value.wrapping_sub(1);
                    self.bus.mem_write(addr, value);
                    self.compare(mode, self.register_a);
                }

                "*ISB" => {
                    let addr = self.get_operand_address(mode);
                    let mut value = self.bus.mem_read(addr);
                    value = value.wrapping_add(1);
                    self.bus.mem_write(addr, value);
                    self.sbc(&opcode_ref.mode); 
                }
                
                "*LAR" => {
                    let value = self.get_operand(mode);
                    let result = self.stack_pointer & value;
                    self.register_a = result;
                    self.register_x = result;
                    self.stack_pointer = result;
                    self.update_zero_and_negative_flags(result);
                }

                "*LAX" => {
                    let value = self.get_operand(mode);
                    self.register_a = value;
                    self.register_x = value;
                    self.update_zero_and_negative_flags(self.register_a);
                }

                "*RLA" => {
                    let addr = self.get_operand_address(mode);
                    let mut data = self.bus.mem_read(addr);
                    let carry = self.get_flag(CARRY_FLAG);
                    self.set_flag(CARRY_FLAG, (data & 0x80) != 0);
                    data <<= 1;
                    if carry {
                        data |= 1;
                    }
                    self.bus.mem_write(addr, data);
                    self.register_a &= data;
                    self.update_zero_and_negative_flags(self.register_a);
                }

                "*RRA" => {
                    let addr = self.get_operand_address(mode);
                    let mut data = self.bus.mem_read(addr);
                    let carry = self.get_flag(CARRY_FLAG);
                    self.set_flag(CARRY_FLAG, (data & 0x01) != 0);
                    data >>= 1;
                    if carry {
                        data |= 0x80;
                    }
                    self.bus.mem_write(addr, data);
                    self.adc(&opcode_ref.mode); 
                }
                
                "*SLO" => {
                    let addr = self.get_operand_address(mode);
                    let mut data = self.bus.mem_read(addr);
                    self.set_flag(CARRY_FLAG, (data & 0x80) != 0);
                    data <<= 1;
                    self.bus.mem_write(addr, data);
                    self.register_a |= data;
                    self.update_zero_and_negative_flags(self.register_a);
                }

                "*SRE" => {
                    let addr = self.get_operand_address(mode);
                    let mut data = self.bus.mem_read(addr);
                    self.set_flag(CARRY_FLAG, (data & 0x01) != 0);
                    data >>= 1;
                    self.bus.mem_write(addr, data);
                    self.register_a ^= data;
                    self.update_zero_and_negative_flags(self.register_a);
                }

                "*SXA" => {
                    let addr = self.get_operand_address(mode);
                    let high = (addr >> 8) as u8;
                    let value = self.register_x & high.wrapping_add(1);
                    self.bus.mem_write(addr, value);
                }

                "*SYA" => {
                    let addr = self.get_operand_address(mode);
                    let high = (addr >> 8) as u8;
                    let value = self.register_y & high.wrapping_add(1);
                    self.bus.mem_write(addr, value);
                }

                "*XAA" => {
                    let value = self.get_operand(mode);
                    self.register_a &= self.register_x & value;
                    self.update_zero_and_negative_flags(self.register_a);
                }

                "*XAS" => {
                    self.stack_pointer = self.register_a & self.register_x;
                    let addr = self.get_operand_address(mode);
                    let high = (addr >> 8) as u8;
                    let value = self.stack_pointer & high.wrapping_add(1);
                    self.bus.mem_write(addr, value);
                }
                _ => todo!(),
            }
            self.bus.tick(opcode_ref.cycles as usize);

            if pc_state == self.program_counter {
                self.program_counter += opcode_ref.bytes as u16;
            }
        }
    }

    fn interrupt_nmi(&mut self){
        self.stack_push_u16(self.program_counter);
        let mut status = self.status;
        status &= !BREAK_COMMAND;
        status |= BREAK_COMMAND_2;
        self.stack_push(status);
        
        self.set_flag(INTERRUPT_DISABLE, true);

        self.program_counter = self.bus.mem_read_u16(0xFFFA);
    }

    pub fn trace(&mut self) -> String {
        let opcodes: HashMap<u8, &'static OpCode> =
            CPU_OPCODES.iter().map(|op| (op.code, op)).collect();

        let code = self.bus.mem_read(self.program_counter);
        let opcode = opcodes.get(&code).unwrap();
        let pc = self.program_counter;

        // 1. Format the instruction bytes (hex dump)
        let mut hex_dump = vec![code];
        if opcode.bytes > 1 {
            hex_dump.push(self.bus.mem_read(pc + 1));
        }
        if opcode.bytes > 2 {
            hex_dump.push(self.bus.mem_read(pc + 2));
        }
        let hex_str = hex_dump
            .iter()
            .map(|z| format!("{:02X}", z))
            .collect::<Vec<String>>()
            .join(" ");

        // 2. Format the assembly instruction string based on addressing mode
        let asm_str = match opcode.mode {
            AddressingMode::Immediate => format!("{} #${:02X}", opcode.name, hex_dump[1]),
            AddressingMode::ZeroPage => {
                let addr = self.bus.mem_read(pc + 1);
                let value = self.bus.mem_read(addr as u16);
                format!("{} ${:02X} = {:02X}", opcode.name, addr, value)
            }
            AddressingMode::ZeroPage_X => {
                let base = self.bus.mem_read(pc + 1);
                let addr = base.wrapping_add(self.register_x);
                let value = self.bus.mem_read(addr as u16);
                format!("{} ${:02X},X @ {:02X} = {:02X}", opcode.name, base, addr, value)
            }
            AddressingMode::ZeroPage_Y => {
                let base = self.bus.mem_read(pc + 1);
                let addr = base.wrapping_add(self.register_y);
                let value = self.bus.mem_read(addr as u16);
                format!("{} ${:02X},Y @ {:02X} = {:02X}", opcode.name, base, addr, value)
            }
            AddressingMode::Absolute => {
                let addr = self.bus.mem_read_u16(pc + 1);
                if opcode.name == "JMP" || opcode.name == "JSR" {
                    format!("{} ${:04X}", opcode.name, addr)
                } else {
                    let value = self.bus.mem_read(addr);
                    format!("{} ${:04X} = {:02X}", opcode.name, addr, value)
                }
            }
            AddressingMode::Absolute_X => {
                let base = self.bus.mem_read_u16(pc + 1);
                let addr = base.wrapping_add(self.register_x as u16);
                let value = self.bus.mem_read(addr);
                format!("{} ${:04X},X @ {:04X} = {:02X}", opcode.name, base, addr, value)
            }
            AddressingMode::Absolute_Y => {
                let base = self.bus.mem_read_u16(pc + 1);
                let addr = base.wrapping_add(self.register_y as u16);
                let value = self.bus.mem_read(addr);
                format!("{} ${:04X},Y @ {:04X} = {:02X}", opcode.name, base, addr, value)
            }
            AddressingMode::Indirect => {
                let ptr_addr = self.bus.mem_read_u16(pc + 1);
                // Replicate the 6502 bug for indirect JMP
                let final_addr = if ptr_addr & 0x00FF == 0x00FF {
                    let lo = self.bus.mem_read(ptr_addr);
                    let hi = self.bus.mem_read(ptr_addr & 0xFF00);
                    (hi as u16) << 8 | (lo as u16)
                } else {
                    self.bus.mem_read_u16(ptr_addr)
                };
                format!("{} (${:04X}) = {:04X}", opcode.name, ptr_addr, final_addr)
            }
            // THIS IS THE KEY FIX from our previous discussion
            AddressingMode::Indirect_X => {
                let base = self.bus.mem_read(pc + 1);
                let ptr = base.wrapping_add(self.register_x);
                // Correctly handle the zero-page wraparound for the address lookup
                let lo = self.bus.mem_read(ptr as u16);
                let hi = self.bus.mem_read(ptr.wrapping_add(1) as u16);
                let addr = (hi as u16) << 8 | (lo as u16);
                let value = self.bus.mem_read(addr);
                format!("{} (${:02X},X) @ {:02X} = {:04X} = {:02X}", opcode.name, base, ptr, addr, value)
            }
            AddressingMode::Indirect_Y => {
                let base = self.bus.mem_read(pc + 1);
                // Correctly handle the zero-page wraparound for the address lookup
                let lo = self.bus.mem_read(base as u16);
                let hi = self.bus.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let addr = deref_base.wrapping_add(self.register_y as u16);
                let value = self.bus.mem_read(addr);
                format!("{} (${:02X}),Y = {:04X} @ {:04X} = {:02X}", opcode.name, base, deref_base, addr, value)
            }
            AddressingMode::Relative => {
                let offset = self.bus.mem_read(pc + 1) as i8;
                let addr = pc.wrapping_add(2).wrapping_add(offset as u16);
                format!("{} ${:04X}", opcode.name, addr)
            }
            // Use opcode.name directly, but handle "ASL A" case for nestest.log
            AddressingMode::Accumulator => format!("{} A", opcode.name),
            AddressingMode::Implied => format!("{}", opcode.name),
        };

        // 3. Combine everything into the final, correctly formatted log line
        format!(
            "{:04X}  {:8} {:<32} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.program_counter,
            hex_str,
            asm_str,
            self.register_a,
            self.register_x,
            self.register_y,
            self.status,
            self.stack_pointer
        )
        .trim_end()
        .to_string()
    }
}

