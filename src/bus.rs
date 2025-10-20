use crate::cartridge::Rom;
use crate::ppu::NesPPU;
use crate::joypad::Joypad; // NEW: Import the Joypad

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
    joypad1: Joypad,
    gameloop_callback: Box<dyn FnMut(&NesPPU, &mut Joypad) + 'call>,
}

impl<'call> Bus<'call> {
    // CHANGED: Update the new function and its signature
    pub fn new<F>(rom: Rom, gameloop_callback: F) -> Self
    where
        F: FnMut(&NesPPU, &mut Joypad) + 'call,
    {
        let ppu = NesPPU::new(rom.chr_rom.clone(), rom.screen_mirroring.clone());
        Bus {
            cpu_vram: [0; 2048],
            rom,
            ppu,
            cycles: 0,
            nmi_interrupt: None,
            joypad1: Joypad::new(), // NEW: Initialize the joypad
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    pub fn dma_transfer(&mut self, page: u8) {
        let mut data = [0u8; 256];
        let start_addr = (page as u16) << 8;
        for i in 0..256 {
            data[i] = self.mem_read(start_addr + i as u16);
        }
        self.ppu.write_oam_dma(&data);
        
        // DMA transfer is not instant. It stalls the CPU for 513 or 514 cycles.
        // We'll use 513 for simplicity.
        self.tick(513);
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.rom.prg_rom[addr as usize]
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycles += cycles as usize;

        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles * 3);
        let nmi_after = self.ppu.nmi_interrupt.is_some();

        if !nmi_before && nmi_after {
            self.nmi_interrupt = Some(1);
            (self.gameloop_callback)(&self.ppu, &mut self.joypad1);
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }
    
    pub fn mem_read_u16_readonly(&self, pos: u16) -> u16 {
        let lo = self.mem_read_readonly(pos) as u16;
        let hi = self.mem_read_readonly(pos + 1) as u16;
        (hi << 8) | lo
    }
    
    pub fn mem_read_readonly(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2002 => self.ppu.status.bits(),

            0x4016 => self.joypad1.peek(), 
            
            0x8000..=0xFFFF => self.read_prg_rom(addr),
            _ => 0,
        }
    }
}

impl<'a> Mem for Bus<'a> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0x2007;
                match mirror_down_addr {
                    0x2002 => self.ppu.read_status(),
                    0x2007 => self.ppu.read_data(),
                    _ => 0,
                }
            }

            0x4016 => self.joypad1.read(),
            0x4017 => 0, 
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
            0x2000..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0x2007;
                match mirror_down_addr {
                    0x2000 => self.ppu.write_to_ctrl(data),
                    0x2001 => self.ppu.write_to_mask(data),
                    0x2003 => self.ppu.write_to_oam_addr(data),
                    0x2004 => self.ppu.write_to_oam_data(data),
                    0x2005 => self.ppu.write_to_scroll(data),
                    0x2006 => self.ppu.write_to_ppu_addr(data),
                    0x2007 => self.ppu.write_to_data(data),
                    _ => { /* Unimplemented */ }
                }
            }
            0x4014 => self.dma_transfer(data), // ADD THIS LINE
            0x4016 => self.joypad1.write(data),
            0x8000..=0xFFFF => { /* Cannot write to ROM */ }
            _ => { /* Ignoring write */ }
        }
    }
}
