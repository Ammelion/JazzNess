use lazy_static::lazy_static;
use std::collections::HashMap;

// Status register flags
const CARRY_FLAG: u8 = 0b0000_0001;
const ZERO_FLAG: u8 = 0b0000_0010;
const INTERRUPT_DISABLE: u8 = 0b0000_0100;
const DECIMAL_MODE: u8 = 0b0000_1000;
const BREAK_COMMAND: u8 = 0b0001_0000;
const UNUSED_FLAG: u8 = 0b0010_0000;
const OVERFLOW_FLAG: u8 = 0b0100_0000;
const NEGATIVE_FLAG: u8 = 0b1000_0000;


#[derive(Debug, Copy, Clone, PartialEq)]
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
   Accumulator,
   NoneAddressing,
}
pub struct CPU{
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    memory: [u8; 0xFFFF],
    opcodes: HashMap<u8, &'static OpCode>,
}

pub struct OpCode {
    pub code: u8,
    pub name: &'static str,
    pub bytes: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
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
        // --- LDA ---
        OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xB1, "LDA", 2, 5, AddressingMode::Indirect_Y),

        // --- LDX ---
        OpCode::new(0xA2, "LDX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA6, "LDX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB6, "LDX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xAE, "LDX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBE, "LDX", 3, 4, AddressingMode::Absolute_Y),

        // --- LDY ---
        OpCode::new(0xA0, "LDY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA4, "LDY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB4, "LDY", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAC, "LDY", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBC, "LDY", 3, 4, AddressingMode::Absolute_X),

        // --- STA ---
        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8D, "STA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9D, "STA", 3, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),

        // --- STX ---
        OpCode::new(0x86, "STX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x96, "STX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x8E, "STX", 3, 4, AddressingMode::Absolute),

        // --- STY ---
        OpCode::new(0x84, "STY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x94, "STY", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8C, "STY", 3, 4, AddressingMode::Absolute),

        // --- Register Transfers ---
        OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xA8, "TAY", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x8A, "TXA", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x98, "TYA", 1, 2, AddressingMode::NoneAddressing),

        // --- Stack Operations ---
        OpCode::new(0x48, "PHA", 1, 3, AddressingMode::NoneAddressing),
        OpCode::new(0x68, "PLA", 1, 4, AddressingMode::NoneAddressing),
        OpCode::new(0x08, "PHP", 1, 3, AddressingMode::NoneAddressing),
        OpCode::new(0x28, "PLP", 1, 4, AddressingMode::NoneAddressing),

        // --- INC (Increment Memory) ---
        OpCode::new(0xE6, "INC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xF6, "INC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xEE, "INC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xFE, "INC", 3, 7, AddressingMode::Absolute_X),

        // --- DEC (Decrement Memory) ---
        OpCode::new(0xC6, "DEC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xD6, "DEC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xCE, "DEC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xDE, "DEC", 3, 7, AddressingMode::Absolute_X),

        // --- ASL ---
        OpCode::new(0x0A, "ASL", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1E, "ASL", 3, 7, AddressingMode::Absolute_X),

        // --- LSR ---
        OpCode::new(0x4A, "LSR", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4E, "LSR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5E, "LSR", 3, 7, AddressingMode::Absolute_X),

        // --- ROL (Rotate Left) ---
        OpCode::new(0x2A, "ROL", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2E, "ROL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3E, "ROL", 3, 7, AddressingMode::Absolute_X),

        // --- ROR (Rotate Right) ---
        OpCode::new(0x6A, "ROR", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6E, "ROR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7E, "ROR", 3, 7, AddressingMode::Absolute_X),

        // --- ADC (Add with Carry) ---
        OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7D, "ADC", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x79, "ADC", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x71, "ADC", 2, 5, AddressingMode::Indirect_Y),

        // --- SBC (Subtract with Carry) ---
        OpCode::new(0xE9, "SBC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE5, "SBC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xF5, "SBC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xED, "SBC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xFD, "SBC", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xF9, "SBC", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xE1, "SBC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xF1, "SBC", 2, 5, AddressingMode::Indirect_Y),
        
        // --- AND ---
        OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3D, "AND", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x39, "AND", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x31, "AND", 2, 5, AddressingMode::Indirect_Y),

        // --- ORA (Inclusive OR) ---
        OpCode::new(0x09, "ORA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0D, "ORA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1D, "ORA", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x19, "ORA", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x01, "ORA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x11, "ORA", 2, 5, AddressingMode::Indirect_Y),

        // --- EOR (Exclusive OR) ---
        OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x4D, "EOR", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5D, "EOR", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x59, "EOR", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x41, "EOR", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x51, "EOR", 2, 5, AddressingMode::Indirect_Y),
        
        // --- Comparisons ---
        // CMP
        OpCode::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xDD, "CMP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xD9, "CMP", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xC1, "CMP", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xD1, "CMP", 2, 5, AddressingMode::Indirect_Y),
        // CPX
        OpCode::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute),
        // CPY
        OpCode::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xCC, "CPY", 3, 4, AddressingMode::Absolute),

        // --- BIT ---
        OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute),
        
        // --- Jumps & Subroutines ---
        OpCode::new(0x4C, "JMP", 3, 3, AddressingMode::Absolute),
        OpCode::new(0x6C, "JMP", 3, 5, AddressingMode::Indirect),
        OpCode::new(0x20, "JSR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x60, "RTS", 1, 6, AddressingMode::NoneAddressing),

        // --- Branches ---
        OpCode::new(0x90, "BCC", 2, 2, AddressingMode::Relative),
        OpCode::new(0xB0, "BCS", 2, 2, AddressingMode::Relative),
        OpCode::new(0xF0, "BEQ", 2, 2, AddressingMode::Relative),
        OpCode::new(0xD0, "BNE", 2, 2, AddressingMode::Relative),
        OpCode::new(0x30, "BMI", 2, 2, AddressingMode::Relative),
        OpCode::new(0x10, "BPL", 2, 2, AddressingMode::Relative),
        OpCode::new(0x50, "BVC", 2, 2, AddressingMode::Relative),
        OpCode::new(0x70, "BVS", 2, 2, AddressingMode::Relative),

        // --- Status Flag Changes ---
        OpCode::new(0x18, "CLC", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x38, "SEC", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xF8, "SED", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x58, "CLI", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x78, "SEI", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xB8, "CLV", 1, 2, AddressingMode::NoneAddressing),
        
        // --- Other Instructions ---
        OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        OpCode::new(0xC8, "INY", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x88, "DEY", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xCA, "DEX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xEA, "NOP", 1, 2, AddressingMode::NoneAddressing),
    ];
}

impl CPU{
    pub fn new() -> Self {
        let mut opcodes = HashMap::new();
        for op in &*CPU_OPCODES {
            opcodes.insert(op.code, op);
        }
        CPU {
            register_a:0,
            register_x:0,
            register_y:0,
            status:0,
            program_counter:0,
            stack_pointer: 0xFD,
            memory: [0; 0xFFFF],
            opcodes: opcodes,
        }
    }

    // --- Flag helpers ---
    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
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
    
    // --- Load/Store/Transfer Instructions ---
    fn lda(&mut self, mode: AddressingMode) {
       let addr=self.get_operand_address(mode);
       let value=self.mem_read(addr);

       self.register_a=value;
       self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: AddressingMode) {
        let addr= self.get_operand_address(mode);
        let value= self.mem_read(addr);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: AddressingMode) {
        let addr= self.get_operand_address(mode);
        let value= self.mem_read(addr);

        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }
    
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // --- Increment/Decrement Instructions ---
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }
    
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn inc(&mut self, mode: AddressingMode) {
        let addr=self.get_operand_address(mode);
        let mut value = self.mem_read(addr);
        value = value.wrapping_add(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn dec(&mut self, mode: AddressingMode) {
        let addr=self.get_operand_address(mode);
        let mut value=self.mem_read(addr);
        value = value.wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    // --- Shift and Rotate Instructions ---
    fn asl(&mut self, mode: AddressingMode) {
        let mut data = if mode == AddressingMode::Accumulator {
            self.register_a
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_read(addr)
        };

        self.set_flag(CARRY_FLAG, data & 0b1000_0000 != 0);
        data <<= 1;

        if mode == AddressingMode::Accumulator {
            self.register_a = data;
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, data);
        }
        self.update_zero_and_negative_flags(data);
    }

    fn lsr(&mut self, mode: AddressingMode) {
        let mut data = if mode == AddressingMode::Accumulator {
            self.register_a
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_read(addr)
        };

        self.set_flag(CARRY_FLAG, data & 0b0000_0001 != 0);
        data >>= 1;

        if mode == AddressingMode::Accumulator {
            self.register_a = data;
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, data);
        }
        self.update_zero_and_negative_flags(data);
    }

    fn rol(&mut self, mode: AddressingMode) {
        let mut data = if mode == AddressingMode::Accumulator {
            self.register_a
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_read(addr)
        };
        
        let old_carry = self.get_flag(CARRY_FLAG);
        self.set_flag(CARRY_FLAG, data & 0b1000_0000 != 0);
        
        data <<= 1;
        if old_carry {
            data |= 0b0000_0001;
        }

        if mode == AddressingMode::Accumulator {
            self.register_a = data;
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, data);
        }
        self.update_zero_and_negative_flags(data);
    }

    fn ror(&mut self, mode: AddressingMode) {
        let mut data = if mode == AddressingMode::Accumulator {
            self.register_a
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_read(addr)
        };
        
        let old_carry = self.get_flag(CARRY_FLAG);
        self.set_flag(CARRY_FLAG, data & 0b0000_0001 != 0);
        
        data >>= 1;
        if old_carry {
            data |= 0b1000_0000;
        }

        if mode == AddressingMode::Accumulator {
            self.register_a = data;
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, data);
        }
        self.update_zero_and_negative_flags(data);
    }

    // --- Arithmetic and Logical Instructions ---
    fn adc(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let carry = if self.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let sum = self.register_a as u16 + value as u16 + carry as u16;
        self.set_flag(CARRY_FLAG, sum > 0xFF);

        let result = sum as u8;
        
        let overflow = (self.register_a ^ result) & (value ^ result) & 0x80 != 0;
        self.set_flag(OVERFLOW_FLAG, overflow);

        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sbc(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let value = (value as i8).wrapping_neg().wrapping_sub(1) as u8;

        let carry = if self.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let sum = self.register_a as u16 + value as u16 + carry as u16;
        self.set_flag(CARRY_FLAG, sum > 0xFF);

        let result = sum as u8;
        
        let overflow = (self.register_a ^ result) & (value ^ result) & 0x80 != 0;
        self.set_flag(OVERFLOW_FLAG, overflow);

        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn and(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a &= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ora(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a |= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn eor(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a ^= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // --- Comparison and Bit Test ---
    fn compare(&mut self, mode: AddressingMode, register_value: u8) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let result = register_value.wrapping_sub(value);

        self.set_flag(CARRY_FLAG, register_value >= value);
        self.update_zero_and_negative_flags(result);
    }

    fn bit(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        
        self.set_flag(ZERO_FLAG, (self.register_a & value) == 0);
        self.set_flag(NEGATIVE_FLAG, (value & NEGATIVE_FLAG) > 0);
        self.set_flag(OVERFLOW_FLAG, (value & OVERFLOW_FLAG) > 0);
    }
    
    // --- Jumps, Branches, and Subroutines ---
    fn branch(&mut self, condition: bool) {
        if condition {
            let jump_offset = self.mem_read(self.program_counter) as i8;
            self.program_counter = self.program_counter
                .wrapping_add(1)
                .wrapping_add(jump_offset as u16);
        }
    }

    fn jmp(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
    }

    fn jsr(&mut self) {
        self.stack_push_u16(self.program_counter + 1);
        self.program_counter = self.mem_read_u16(self.program_counter);
    }

    fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16() + 1;
    }
    
    // --- Stack Operations ---
    fn stack_push_u8(&mut self, data: u8) {
        self.mem_write(0x0100 + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop_u8(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read(0x0100 + self.stack_pointer as u16)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.stack_push_u8(hi);
        self.stack_push_u8(lo);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop_u8() as u16;
        let hi = self.stack_pop_u8() as u16;
        (hi << 8) | lo
    }

    fn pha(&mut self) {
        self.stack_push_u8(self.register_a);
    }

    fn pla(&mut self) {
        self.register_a = self.stack_pop_u8();
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn php(&mut self) {
        // PHP sets the B flag when pushing.
        self.stack_push_u8(self.status | BREAK_COMMAND | UNUSED_FLAG);
    }

    fn plp(&mut self) {
        self.status = self.stack_pop_u8();
        // PLP ignores the B and Unused flags from the stack.
        self.set_flag(BREAK_COMMAND, false);
        self.set_flag(UNUSED_FLAG, true);
    }

    // --- Memory Access ---
    fn mem_read(&self,addr:u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self,addr:u16,data:u8){
        self.memory[addr as usize] = data;
    }
    
    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
   }

   // --- Core Execution ---
   fn get_operand_address(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
 
            AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
 
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_y) as u16
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_x as u16)
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_y as u16)
            }
            
            AddressingMode::Indirect => {
                 let ptr_addr = self.mem_read_u16(self.program_counter);
                 if ptr_addr & 0x00FF == 0x00FF {
                    let lo = self.mem_read(ptr_addr) as u16;
                    let hi = self.mem_read(ptr_addr & 0xFF00) as u16;
                    (hi << 8) | lo
                 } else {
                     self.mem_read_u16(ptr_addr)
                 }
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                deref_base.wrapping_add(self.register_y as u16)
            }

            // Relative addressing is handled within the branch instructions themselves
            AddressingMode::Relative | AddressingMode::Accumulator | AddressingMode::NoneAddressing => {
               panic!("mode {:?} is not supposed to be handled by get_operand_address", mode);
            }
        }
    }

    pub fn load_and_run(&mut self, program: Vec<u8>){
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn load(&mut self, program: Vec<u8>){
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }
    
    pub fn reset(&mut self) {
       self.register_a = 0;
       self.register_x = 0;
       self.register_y = 0;
       self.stack_pointer = 0xFD;
       self.status = UNUSED_FLAG | INTERRUPT_DISABLE;

       self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn run(&mut self) {
        loop {
            let code = self.mem_read(self.program_counter);
            
            let (name, mode, bytes) = {
                let opcode = self.opcodes.get(&code).expect(&format!("OpCode {:x} is not recognized", code));
                (opcode.name, opcode.mode, opcode.bytes) 
            };

            self.program_counter += 1;
            let pc_state_before_exec = self.program_counter;
            
            match name {
                "LDA" => self.lda(mode),
                "LDX" => self.ldx(mode),
                "LDY" => self.ldy(mode),
                "STA" => self.sta(mode),
                "STX" => self.stx(mode),
                "STY" => self.sty(mode),
                "TAX" => self.tax(),
                "TAY" => self.tay(),
                "TXA" => self.txa(),
                "TYA" => self.tya(),

                // Arithmetic
                "ADC" => self.adc(mode),
                "SBC" => self.sbc(mode),

                // Logical
                "AND" => self.and(mode),
                "ORA" => self.ora(mode),
                "EOR" => self.eor(mode),

                // Compare & Bit
                "CMP" => self.compare(mode, self.register_a),
                "CPX" => self.compare(mode, self.register_x),
                "CPY" => self.compare(mode, self.register_y),
                "BIT" => self.bit(mode),
                
                // Stack
                "PHA" => self.pha(),
                "PLA" => self.pla(),
                "PHP" => self.php(),
                "PLP" => self.plp(),

                // Jumps & Subroutines
                "JMP" => self.jmp(mode),
                "JSR" => self.jsr(),
                "RTS" => self.rts(),
                
                // Branches
                "BCC" => self.branch(!self.get_flag(CARRY_FLAG)),
                "BCS" => self.branch(self.get_flag(CARRY_FLAG)),
                "BEQ" => self.branch(self.get_flag(ZERO_FLAG)),
                "BNE" => self.branch(!self.get_flag(ZERO_FLAG)),
                "BMI" => self.branch(self.get_flag(NEGATIVE_FLAG)),
                "BPL" => self.branch(!self.get_flag(NEGATIVE_FLAG)),
                "BVC" => self.branch(!self.get_flag(OVERFLOW_FLAG)),
                "BVS" => self.branch(self.get_flag(OVERFLOW_FLAG)),

                // Status Flag Changes
                "CLC" => self.set_flag(CARRY_FLAG, false),
                "SEC" => self.set_flag(CARRY_FLAG, true),
                "CLD" => self.set_flag(DECIMAL_MODE, false),
                "SED" => self.set_flag(DECIMAL_MODE, true),
                "CLI" => self.set_flag(INTERRUPT_DISABLE, false),
                "SEI" => self.set_flag(INTERRUPT_DISABLE, true),
                "CLV" => self.set_flag(OVERFLOW_FLAG, false),

                "INX" => self.inx(),
                "INY" => self.iny(),
                "DEY" => self.dey(),
                "DEX" => self.dex(),
                "INC" => self.inc(mode),
                "DEC" => self.dec(mode),
                "ASL" => self.asl(mode),
                "LSR" => self.lsr(mode),
                "ROL" => self.rol(mode),
                "ROR" => self.ror(mode),

                "NOP" => { },
                "BRK" => return,
                _ => todo!("Instruction {} not implemented", name),
            }

            if pc_state_before_exec == self.program_counter {
                self.program_counter += (bytes - 1) as u16;
            }
        }
    }
}

fn main() {
    println!("NES CPU Emulator Core. To run tests, execute 'cargo test'.");
}