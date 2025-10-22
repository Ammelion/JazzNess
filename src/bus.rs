// In src/bus.rs

use crate::apu::Apu;
use crate::cartridge::Rom;
// --- ADD THIS IMPORT ---
use crate::gamegenie::GameGenieCode;
use crate::joypad::Joypad;
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
    pub apu: Apu, // APU struct added to the Bus
    cycles: usize,
    nmi_interrupt: Option<u8>,
    irq_interrupt: Option<u8>,
    pub joypad1: Joypad,
    pub joypad2: Joypad,
    // Gameloop callback signature updated to include Apu
    gameloop_callback: Box<dyn FnMut(&NesPPU, &mut Joypad, &mut Apu) + 'call>,
    
    // --- ADD THIS FIELD ---
    game_genie_codes: Vec<GameGenieCode>,
}

impl<'call> Bus<'call> {
    // CHANGED: Update the new function and its signature to accept the new callback
    pub fn new<F>(rom: Rom, gameloop_callback: F) -> Self
    where
        F: FnMut(&NesPPU, &mut Joypad, &mut Apu) + 'call, // Updated signature
    {
        let ppu = NesPPU::new(rom.chr_rom.clone(), rom.screen_mirroring.clone());
        Bus {
            cpu_vram: [0; 2048],
            rom,
            ppu,
            apu: Apu::new(), // Initialize the APU
            cycles: 0,
            nmi_interrupt: None,
            irq_interrupt: None,
            joypad1: Joypad::new(),
            joypad2: Joypad::new(),
            gameloop_callback: Box::from(gameloop_callback),
            
            // --- INITIALIZE THE NEW FIELD ---
            game_genie_codes: Vec::new(),
        }
    }
    
    // --- ADD THIS NEW PUBLIC METHOD ---
    /// Sets the list of active Game Genie codes.
    pub fn set_game_genie_codes(&mut self, codes: Vec<GameGenieCode>) {
        self.game_genie_codes = codes;
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
    
    // --- THIS IS THE NEWLY RENAMED RAW READ FUNCTION ---
    fn read_prg_rom_raw(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.rom.prg_rom[addr as usize]
    }

    // --- THIS IS THE MODIFIED FUNCTION WITH GAME GENIE LOGIC ---
    fn read_prg_rom(&self, addr: u16) -> u8 {
        // Check for an active Game Genie code at this address
        for code in &self.game_genie_codes {
            if code.address == addr {
                // Address matches. Check if it's a conditional code.
                if let Some(compare_data) = code.compare_data {
                    // It is. We must read the *actual* ROM data to compare.
                    let actual_data = self.read_prg_rom_raw(addr);
                    if actual_data == compare_data {
                        // Condition matches, return the new data
                        return code.new_data;
                    } else {
                        // Condition failed, break from the code loop
                        // and fall through to return the raw ROM data.
                        break;
                    }
                } else {
                    // Not a conditional code. Just return the new data.
                    return code.new_data;
                }
            }
        }

        // No matching/triggered codes. Read from ROM as normal.
        self.read_prg_rom_raw(addr)
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycles += cycles;

        // Clock the APU by the number of CPU cycles
        self.apu.tick(cycles);

        // PPU runs 3x faster than CPU
        let frame_complete = self.ppu.tick(cycles * 3);

        if frame_complete {
            // Pass the APU to the gameloop callback
            (self.gameloop_callback)(&self.ppu, &mut self.joypad1, &mut self.apu);
        }

        if self.ppu.poll_nmi_interrupt().is_some() {
            self.nmi_interrupt = Some(1);
        }

        if self.apu.poll_frame_interrupt() {
            self.irq_interrupt = Some(1);
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }
    
    pub fn poll_irq_status(&mut self) -> Option<u8> { // <--- ADD THIS FUNCTION
        self.irq_interrupt.take()
    }

    pub fn mem_read_readonly(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize]
            }
            // --- MODIFIED TO USE THE PATCHING FUNCTION ---
            0x8000..=0xFFFF => self.read_prg_rom(addr),
            _ => 0,
        }
    }

    pub fn mem_read_u16_readonly(&self, pos: u16) -> u16 {
        let lo = self.mem_read_readonly(pos) as u16;
        let hi = self.mem_read_readonly(pos + 1) as u16;
        (hi << 8) | lo
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

            // APU/Joypad range
            0x4015 => self.apu.mem_read(addr), // APU Status Read
            0x4016 => self.joypad1.read(),
            0x4017 => self.joypad2.read(),
            // End APU/Joypad range

            // --- MODIFIED TO USE THE PATCHING FUNCTION ---
            0x8000..=0xFFFF => self.read_prg_rom(addr),
            _ => 0, // Other APU regs ($4000-$4013) are write-only
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

            // APU/Joypad range
            // Delegate $4000-$4013, $4015, $4017 to the APU
            0x4000..=0x4013 | 0x4015 | 0x4017 => {
                self.apu.mem_write(addr, data);
            }
            0x4014 => self.dma_transfer(data), // OAM DMA
            0x4016 => {
                // Joypad strobe
                self.joypad1.write(data);
                self.joypad2.write(data);
            }
            // End APU/Joypad range

            0x8000..=0xFFFF => { /* Cannot write to ROM */ }
            _ => { /* Ignoring write */ }
        }
    }
}