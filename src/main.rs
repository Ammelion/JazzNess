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
   Indirect_X,
   Indirect_Y,
   Accumulator,
   NoneAddressing,
}
pub struct CPU{
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
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

        // ROL (Rotate Left)
        OpCode::new(0x2A, "ROL", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2E, "ROL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3E, "ROL", 3, 7, AddressingMode::Absolute_X),

        // ROR (Rotate Right)
        OpCode::new(0x6A, "ROR", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6E, "ROR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7E, "ROR", 3, 7, AddressingMode::Absolute_X),

        // --- Other Instructions ---
        OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        OpCode::new(0xC8, "INY", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x88, "DEY", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xCA, "DEX", 1, 2, AddressingMode::NoneAddressing),
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
            memory: [0; 0xFFFF],
            opcodes: opcodes,
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
       let addr=self.get_operand_address(mode);
       let value=self.mem_read(addr);

       self.register_a=value;
       self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr= self.get_operand_address(mode);
        let value= self.mem_read(addr);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr= self.get_operand_address(mode);
        let value= self.mem_read(addr);

        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
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

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr=self.get_operand_address(mode);
        let mut value = self.mem_read(addr);
        value = value.wrapping_add(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr=self.get_operand_address(mode);
        let mut value=self.mem_read(addr);
        value = value.wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn set_carry_flag(&mut self) {
        self.status |= 0b0000_0001;
    }

    fn clear_carry_flag(&mut self) {
        self.status &= 0b1111_1110;
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let mut data = if let AddressingMode::Accumulator = mode {
            self.register_a
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_read(addr)
        };

        if data & 0b1000_0000 != 0 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        data <<= 1;

        if let AddressingMode::Accumulator = mode {
            self.register_a = data;
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, data);
        }
        self.update_zero_and_negative_flags(data);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let mut data = if let AddressingMode::Accumulator = mode {
            self.register_a
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_read(addr)
        };

        if data & 0b0000_0001 != 0 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        data >>= 1;

        if let AddressingMode::Accumulator = mode {
            self.register_a = data;
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, data);
        }
        self.update_zero_and_negative_flags(data);
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let mut data = if let AddressingMode::Accumulator = mode {
            self.register_a
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_read(addr)
        };
        
        let old_carry = self.status & 0b0000_0001 != 0;

        if data & 0b1000_0000 != 0 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        
        data <<= 1;
        if old_carry {
            data |= 0b0000_0001;
        }

        if let AddressingMode::Accumulator = mode {
            self.register_a = data;
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, data);
        }
        self.update_zero_and_negative_flags(data);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let mut data = if let AddressingMode::Accumulator = mode {
            self.register_a
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_read(addr)
        };
        
        let old_carry = self.status & 0b0000_0001 != 0;

        if data & 0b0000_0001 != 0 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }
        
        data >>= 1;
        if old_carry {
            data |= 0b1000_0000;
        }

        if let AddressingMode::Accumulator = mode {
            self.register_a = data;
        } else {
            let addr = self.get_operand_address(mode);
            self.mem_write(addr, data);
        }
        self.update_zero_and_negative_flags(data);
    }

    fn mem_read(&self,addr:u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self,addr:u16,data:u8){
        self.memory[addr as usize] = data;
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

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
   }

   fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
 
            AddressingMode::ZeroPage  => self.mem_read(self.program_counter) as u16,
 
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
               deref
            }

            AddressingMode::Accumulator | AddressingMode::NoneAddressing => {
               panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn reset(&mut self) {
       self.register_a = 0;
       self.register_x = 0;
       self.register_y = 0;
       self.status = 0b0010_0100;

       self.program_counter = self.mem_read_u16(0xFFFC);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result==0{
            self.status=self.status | 0b0000_0010;
        }
        else{
            self.status=self.status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }
    pub fn run(&mut self) {
        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;

            let opcode = self.opcodes.get(&code).expect(&format!("OpCode {:x} is not recognized", code));

            let mode = &opcode.mode;
            let instruction_name = opcode.name;
            let bytes_to_add = (opcode.bytes - 1) as u16;

            match instruction_name {
                "LDA" =>self.lda(mode),

                "LDX" =>self.ldx(mode),
                
                "LDY" =>self.ldy(mode),
                
                "STA" =>self.sta(mode),

                "STX" =>self.stx(mode),

                "STY" =>self.sty(mode),

                "TAX" =>self.tax(),

                "TAY" => self.tay(),

                "TXA" => self.txa(),

                "TYA" => self.tya(),

                "INX" =>self.inx(),

                "INY" => self.iny(),

                "DEY" => self.dey(),

                "DEX" => self.dex(),

                "INC" => self.inc(mode),

                "DEC" => self.dec(mode),

                "ASL" => self.asl(mode),

                "LSR" => self.lsr(mode),

                "ROL" => self.rol(mode),

                "ROR" => self.ror(mode),

                "BRK" =>return,

                _ => todo!(),
            }

            self.program_counter += bytes_to_add;
        }
    }
}

fn main() {
    println!("Hello, world!");
}