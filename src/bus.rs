use crate::cartridge::Rom;
use crate::ppu::NesPPU;
pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
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
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    rom: Rom,
    ppu: NesPPU,
    cycles: usize,
    nmi_interrupt: Option<u8>,
    gameloop_callback: Box<dyn FnMut(&NesPPU) + 'call>,
}

impl<'call> Bus<'call> {
    pub fn new<F>(rom: Rom, gameloop_callback: F) -> Self
    where
        F: FnMut(&NesPPU) + 'call,
    {
        let ppu = NesPPU::new(rom.chr_rom.clone(), rom.screen_mirroring.clone());
        Bus {
            cpu_vram: [0; 2048],
            rom,
            ppu,
            cycles: 0,
            nmi_interrupt: None,
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.rom.prg_rom[addr as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles * 3);
        let nmi_after = self.ppu.nmi_interrupt.is_some();

        if !nmi_before && nmi_after {
            self.nmi_interrupt = Some(1); // <-- Set the flag for the CPU
            (self.gameloop_callback)(&self.ppu);
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }
}

impl<'a> Mem for Bus<'a> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize]
            }
            // PPU Registers range
            0x2000..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0x2007;
                match mirror_down_addr {
                    // Reads from most PPU registers are not allowed or have no effect.
                    // Reading from $2002 (Status) will be implemented later.
                    0x2000 | 0x2001 | 0x2003 | 0x2004 | 0x2005 | 0x2006 | 0x4014 => 0,
                    0x2002 => self.ppu.read_status(),
                    0x2007 => self.ppu.read_data(),
                    _ => 0,
                }
            }
            // PRG ROM
            0x8000..=0xFFFF => self.read_prg_rom(addr),
            _ => 0,
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            // PPU Registers range
            0x2000..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0x2007;
                match mirror_down_addr {
                    0x2000 => self.ppu.write_to_ctrl(data),
                    0x2001 => { /* todo!("PPU Mask Register") */ }
                    0x2002 => panic!("Attempt to write to read-only PPU status register"),
                    0x2003 => { /* todo!("PPU OAM Address Register") */ }
                    0x2004 => { /* todo!("PPU OAM Data Register") */ }
                    0x2005 => { /* todo!("PPU Scroll Register") */ }
                    0x2006 => self.ppu.write_to_ppu_addr(data),
                    0x2007 => self.ppu.write_to_data(data),
                    _ => unreachable!(),
                }
            }
            // PRG ROM (writing is disallowed)
            0x8000..=0xFFFF => {
            }
            _ => { /* Ignoring write */ }
        }
    }
}