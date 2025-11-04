// In src/ppu.rs

use crate::cartridge::Mirroring;
use bitflags::bitflags;

bitflags! {
    pub struct ControlRegister: u8 {
        const NAMETABLE1              = 0b0000_0001;
        const NAMETABLE2              = 0b0000_0010;
        const VRAM_ADD_INCREMENT      = 0b0000_0100;
        const SPRITE_PATTERN_ADDR     = 0b0000_1000;
        const BACKROUND_PATTERN_ADDR  = 0b0001_0000;
        const SPRITE_SIZE             = 0b0010_0000;
        const MASTER_SLAVE_SELECT     = 0b0100_0000;
        const GENERATE_NMI            = 0b1000_0000;
    }
}

bitflags! {
    pub struct MaskRegister: u8 {
        const GREYSCALE               = 0b0000_0001;
        const LEFTMOST_BG             = 0b0000_0010; 
        const LEFTMOST_SPRITES        = 0b0000_0100; 
        const SHOW_BACKGROUND         = 0b0000_1000;
        const SHOW_SPRITES            = 0b0001_0000;
        const EMPHASIZE_RED           = 0b0010_0000;
        const EMPHASIZE_GREEN         = 0b0100_0000;
        const EMPHASIZE_BLUE          = 0b1000_0000;
    }
}

bitflags! {
    pub struct StatusRegister: u8 {
        const SPRITE_OVERFLOW   = 0b0010_0000;
        const SPRITE_0_HIT      = 0b0100_0000;
        const VBLANK_STARTED    = 0b1000_0000;
    }
}

#[derive(Default)]
pub struct ScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    write_latch: bool,
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn update(&mut self, data: u8) {
        *self = ControlRegister::from_bits_truncate(data);
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1 
        } else {
            32
        }
    }

    pub fn sprite_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::SPRITE_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }

    pub fn background_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::BACKROUND_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }

    pub fn nametable_addr(&self) -> u16 {
        match self.bits() & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        }
    }
}

pub struct AddrRegister {
    value: u16,
    write_latch: bool, 
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: 0,
            write_latch: false,
        }
    }

    fn set(&mut self, data: u16) {
        self.value = data & 0x3FFF;
    }

    pub fn update(&mut self, data: u8) {
        if !self.write_latch {
            self.value = (self.value & 0x00FF) | ((data as u16) << 8);
        } else {
            self.value = (self.value & 0xFF00) | (data as u16);
        }

        self.set(self.value);

        self.write_latch = !self.write_latch;
    }

    pub fn increment(&mut self, inc: u8) {
        self.value = self.value.wrapping_add(inc as u16);
        self.set(self.value); 
    }

    pub fn reset_latch(&mut self) {
        self.write_latch = false;
    }

    pub fn get(&self) -> u16 {
        self.value
    }
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            write_latch: false,
        }
    }

    pub fn write(&mut self, data: u8) {
        if !self.write_latch {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.write_latch = !self.write_latch;
    }

    pub fn reset_latch(&mut self) {
        self.write_latch = false;
    }
}

pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring,
    pub ctrl: ControlRegister,
    pub mask: MaskRegister,
    pub status: StatusRegister,
    pub scroll: ScrollRegister,

    pub vram: [u8; 2048],
    pub oam_addr: u8,
    pub oam_data: [u8; 256],
    pub palette_table: [u8; 32],

    addr: AddrRegister,
    internal_data_buf: u8,

    scanline: u16,
    cycles: usize,
    pub nmi_interrupt: Option<u8>, 
}

impl NesPPU {

    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom,
            mirroring,
            ctrl: ControlRegister::new(),
            mask: MaskRegister::from_bits_truncate(0),
            status: StatusRegister::from_bits_truncate(0),
            scroll: ScrollRegister::new(),
            vram: [0; 2048],
            oam_addr: 0,
            oam_data: [0; 256],
            palette_table: [0; 32],
            addr: AddrRegister::new(),
            internal_data_buf: 0,
            scanline: 0,
            cycles: 0,
            nmi_interrupt: None,
        }
    }

    pub fn tick(&mut self, cycles: usize) -> bool {
        self.cycles += cycles;
        if self.scanline < 240 && self.cycles >= 1 && self.cycles <= 256 {
            if self.mask.contains(MaskRegister::SHOW_BACKGROUND | MaskRegister::SHOW_SPRITES) {
                if !self.status.contains(StatusRegister::SPRITE_0_HIT) {
                    let y = self.oam_data[0] as usize;
                    let x = self.oam_data[3] as usize;
                    if y == self.scanline as usize && x == self.cycles {
                        let bg_clipped = !self.mask.contains(MaskRegister::LEFTMOST_BG);
                        let sp_clipped = !self.mask.contains(MaskRegister::LEFTMOST_SPRITES);
                        let in_clip_region = self.cycles < 8 && (bg_clipped || sp_clipped);
                        if !in_clip_region && self.cycles != 255 {
                           self.status.insert(StatusRegister::SPRITE_0_HIT);
                        }
                    }
                }
            }
        }
        if self.cycles >= 341 {
            self.cycles %= 341; 
            self.scanline += 1; 

            if self.scanline == 241 {
                self.status.insert(StatusRegister::VBLANK_STARTED);
                if self.ctrl.contains(ControlRegister::GENERATE_NMI) {
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status.remove(StatusRegister::VBLANK_STARTED);
                self.status.remove(StatusRegister::SPRITE_0_HIT);
                self.status.remove(StatusRegister::SPRITE_OVERFLOW);
                self.nmi_interrupt = None;
                
                return true; 
            }
        }
        false 
    }

    pub fn poll_nmi_interrupt(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }


    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_enabled = self.ctrl.contains(ControlRegister::GENERATE_NMI);
        self.ctrl.update(value);
        let after_nmi_enabled = self.ctrl.contains(ControlRegister::GENERATE_NMI);

        if !before_nmi_enabled && after_nmi_enabled && self.status.contains(StatusRegister::VBLANK_STARTED) {
            self.nmi_interrupt = Some(1);
        }
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.mask = MaskRegister::from_bits_truncate(value);
    }

    pub fn read_status(&mut self) -> u8 {
        let data = self.status.bits();
        self.status.remove(StatusRegister::VBLANK_STARTED);
        self.addr.reset_latch();
        self.scroll.reset_latch();
        data
    }
    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        // OAM address increments after write
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }
    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
        for &byte in data.iter() {
            self.oam_data[self.oam_addr as usize] = byte;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll.write(value);
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();

        match addr {
            0..=0x1FFF => {
                eprintln!("Warning: Attempted write to CHR address {:#X}", addr);
            }
            0x2000..=0x3EFF => {
                let mirrored_addr = self.mirror_vram_addr(addr);
                self.vram[mirrored_addr as usize] = value;
            }
            0x3F00..=0x3FFF => {
                let mirrored_addr = addr & 0x3F1F;
                let mut palette_addr = (mirrored_addr - 0x3F00) as usize;

                if palette_addr == 0x10 || palette_addr == 0x14 || palette_addr == 0x18 || palette_addr == 0x1C {
                    palette_addr -= 0x10;
                }
                self.palette_table[palette_addr] = value;
            }
            _ => unreachable!(),
        }

        self.increment_vram_addr();
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();

        self.increment_vram_addr();

        match addr {
            0..=0x3EFF => {
                let buffered_data = self.internal_data_buf;

                self.internal_data_buf = match addr {
                    0..=0x1FFF => self.chr_rom[addr as usize],
                    0x2000..=0x3EFF => {
                        let mirrored_addr = self.mirror_vram_addr(addr);
                        self.vram[mirrored_addr as usize]
                    }
                    _ => unreachable!(),
                };
                buffered_data
            }

            0x3F00..=0x3FFF => {
                let mirrored_addr = addr & 0x3F1F;
                let mut palette_addr = (mirrored_addr - 0x3F00) as usize;
                let underlying_vram_addr = self.mirror_vram_addr(addr);
                self.internal_data_buf = self.vram[underlying_vram_addr as usize];

                if palette_addr == 0x10 || palette_addr == 0x14 || palette_addr == 0x18 || palette_addr == 0x1C {
                    palette_addr -= 0x10;
                }
                self.palette_table[palette_addr]
            }
            _ => unreachable!(),
        }
    }


    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0x2FFF; 
        let vram_index = mirrored_vram - 0x2000; 
        let name_table = vram_index / 0x400;

        match self.mirroring {
            Mirroring::VERTICAL => match name_table {
                0 | 2 => vram_index & 0x3FF,
                1 | 3 => (vram_index & 0x3FF) + 0x400,
                _ => unreachable!(),
            },
            Mirroring::HORIZONTAL => match name_table {
                0 | 1 => vram_index & 0x3FF,
                2 | 3 => (vram_index & 0x3FF) + 0x400, 
                _ => unreachable!(),
            },
            Mirroring::FOURSCREEN => vram_index & 0x3FF,
        }
    }

    pub fn peek_status(&self) -> u8 {
        self.status.bits()
    }
}