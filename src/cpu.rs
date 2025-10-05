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

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub stack_pointer: u8,
    pub status: u8,
    pub program_counter: u16,
    bus: Bus,
}

pub struct OpCode {
    pub code: u8,
    pub name: &'static str,
    pub bytes: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }
}

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
    ];
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
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

    // --- Private Helper Methods that now use the Bus via the Mem trait ---
    fn stack_push(&mut self, data: u8) {
        self.mem_write(0x0100 + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn stack_pull(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read(0x0100 + self.stack_pointer as u16)
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

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPage_X => self.mem_read(self.program_counter).wrapping_add(self.register_x) as u16,
            AddressingMode::ZeroPage_Y => self.mem_read(self.program_counter).wrapping_add(self.register_y) as u16,
            AddressingMode::Absolute_X => self.mem_read_u16(self.program_counter).wrapping_add(self.register_x as u16),
            AddressingMode::Absolute_Y => self.mem_read_u16(self.program_counter).wrapping_add(self.register_y as u16),
            AddressingMode::Indirect => {
                let ptr = self.mem_read_u16(self.program_counter);
                if ptr & 0x00FF == 0x00FF {
                    let lo = self.mem_read(ptr);
                    let hi = self.mem_read(ptr & 0xFF00);
                    (hi as u16) << 8 | lo as u16
                } else {
                    self.mem_read_u16(ptr)
                }
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | lo as u16
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | lo as u16;
                deref_base.wrapping_add(self.register_y as u16)
            }
            AddressingMode::Relative => {
                let offset = self.mem_read(self.program_counter) as i8;
                self.program_counter.wrapping_add(1).wrapping_add(offset as u16)
            }
            AddressingMode::Implied | AddressingMode::Accumulator => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn get_operand(&self, mode: &AddressingMode) -> u8 {
        match mode {
            AddressingMode::Accumulator => self.register_a,
            _ => self.mem_read(self.get_operand_address(mode)),
        }
    }

    fn set_operand(&mut self, mode: &AddressingMode, value: u8) {
        match mode {
            AddressingMode::Accumulator => self.register_a = value,
            _ => {
                let addr = self.get_operand_address(mode);
                self.mem_write(addr, value);
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
        self.program_counter = self.mem_read_u16(0xFFFC);
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
        let opcodes: HashMap<u8, &'static OpCode> =
            CPU_OPCODES.iter().map(|op| (op.code, op)).collect();

        loop {
            callback(self);
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let pc_state = self.program_counter;

            let opcode_ref = opcodes
                .get(&code)
                .unwrap_or_else(|| panic!("OpCode {:x} is not recognized", code));

            let mode = &opcode_ref.mode;
            let name = opcode_ref.name;
            let bytes = opcode_ref.bytes;

            match name {
                "BRK" => return,
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
                    self.stack_push_u16(self.program_counter + 2 - 1);
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

                _ => todo!(),
            }

            if pc_state == self.program_counter {
                self.program_counter += (bytes - 1) as u16;
            }
        }
    }
}
