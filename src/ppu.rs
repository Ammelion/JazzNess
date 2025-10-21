use crate::cartridge::Mirroring;
use bitflags::bitflags;

bitflags! {
    // 7  bit  0
    // ---- ----
    // VPHB SINN
    // |||| ||||
    // |||| ||++- Base nametable address
    // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
    // |||| |     (0: add 1, going across; 1: add 32, going down)
    // |||| +---- Sprite pattern table address for 8x8 sprites
    // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
    // |||+------ Background pattern table address (0: $0000; 1: $1000)
    // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    // |+-------- PPU master/slave select
    // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
    // +--------- Generate an NMI at the start of the
    //            vertical blanking interval (0: off; 1: on)
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

    pub struct StatusRegister: u8 {
        const VBLANK_STARTED    = 0b1000_0000;
        const SPRITE_0_HIT      = 0b0100_0000;
        const SPRITE_OVERFLOW   = 0b0010_0000;
    }
}

pub struct ScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    latch: bool,
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
            0
        } else {
            0x1000
        }
    }

    pub fn background_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::BACKROUND_PATTERN_ADDR) { 0 } else { 0x1000 }
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
    value: (u8, u8), // (hi, lo)
    hi_ptr: bool,
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0),
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xff) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        // Mirror down addresses greater than 0x3FFF
        if self.get() > 0x3FFF {
            self.set(self.get() & 0x3FFF);
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        if self.get() > 0x3FFF {
            self.set(self.get() & 0x3FFF);
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            latch: false,
        }
    }

    pub fn write(&mut self, data: u8) {
        if !self.latch {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.latch = !self.latch;
    }

    pub fn reset_latch(&mut self) {
        self.latch = false;
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

    // In ammelion/jazzness/.../src/ppu.rs
    // Replace your entire existing tick() function with this one.
    pub fn tick(&mut self, cycles: usize) -> bool {
        self.cycles += cycles;

        // --- START NEW LOGIC ---
        // Perform Sprite 0 Hit detection
        // This must happen on visible scanlines (0-239) and visible cycles (1-256)
        if self.scanline < 240 && self.cycles >= 1 && self.cycles <= 256 {
            // Check if rendering is enabled
            if self.mask.contains(MaskRegister::SHOW_BACKGROUND) && 
            self.mask.contains(MaskRegister::SHOW_SPRITES) 
            {
                // Only set the flag if it's not already set
                if !self.status.contains(StatusRegister::SPRITE_0_HIT) {
                    // Check for hit at the *current* cycle
                    // Your is_sprite_0_hit function is:
                    // (y == self.scanline) && (x <= cycle)
                    // We will call it with the current cycle.
                    // Note: A more accurate PPU would check x == cycle, but
                    // your function's (x <= cycle) will work for now.
                    
                    let y = self.oam_data[0] as usize;
                    let x = self.oam_data[3] as usize;
                    
                    // We check for hit on this specific cycle.
                    // A hit occurs if:
                    // 1. Sprite 0's Y is on the current scanline.
                    // 2. Sprite 0's X is exactly at the current cycle.
                    // 3. We are not in the first 8 pixels (games mask this).
                    if y == self.scanline as usize && x as usize == self.cycles && self.cycles != 255 {
                        self.status.insert(StatusRegister::SPRITE_0_HIT);
                    }
                }
            }
        }
        // --- END NEW LOGIC ---

        if self.cycles >= 341 {
            self.cycles = self.cycles % 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.status.insert(StatusRegister::VBLANK_STARTED);
                // We NO LONGER clear sprite 0 hit here.
                // The CPU must be able to read it during VBlank.
                // self.status.remove(StatusRegister::SPRITE_0_HIT); 
                
                if self.ctrl.contains(ControlRegister::GENERATE_NMI) {
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = None;
                self.status.remove(StatusRegister::VBLANK_STARTED);
                self.status.remove(StatusRegister::SPRITE_OVERFLOW);
                self.status.remove(StatusRegister::SPRITE_0_HIT); // Clear flag at the *end* of VBlank
                return true; // Frame is complete
            }
        }
        false
    }

    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;
        (y == self.scanline as usize) && x <= cycle && self.mask.contains(MaskRegister::SHOW_SPRITES)
    }

    pub fn poll_nmi_interrupt(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }

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

    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi = self.ctrl.contains(ControlRegister::GENERATE_NMI);
        self.ctrl.update(value);
        let after_nmi = self.ctrl.contains(ControlRegister::GENERATE_NMI);
        
        if !before_nmi && after_nmi && self.status.contains(StatusRegister::VBLANK_STARTED) {
            self.nmi_interrupt = Some(1);
        }
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll.write(value);
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.mask = MaskRegister::from_bits_truncate(value);
    }
    
    // ADD THESE FUNCTIONS FOR OAM
    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }
    
    pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
        for x in data.iter() {
            self.oam_data[self.oam_addr as usize] = *x;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }

    pub fn read_status(&mut self) -> u8 {
        let data = self.status.bits();
        self.status.remove(StatusRegister::VBLANK_STARTED);
        self.addr.reset_latch();
        self.scroll.reset_latch(); 
        data
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    // Horizontal:
    //   [ A ] [ a ]
    //   [ B ] [ b ]
    // Vertical:
    //   [ A ] [ B ]
    //   [ a ] [ b ]
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0x2FFF; // Mirror down 0x3000-0x3EFF to 0x2000-0x2EFF
        let vram_index = mirrored_vram - 0x2000;
        let name_table_index = vram_index / 0x400;

        match (&self.mirroring, name_table_index) {
            (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 1) | (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }
    
    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        match addr {
            0..=0x1FFF => {},
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3EFF => {
                let mirrored_addr = addr - 0x1000;
                self.vram[self.mirror_vram_addr(mirrored_addr) as usize] = value;
            }
            // Palette RAM
            0x3F00..=0x3FFF => {
                let mut palette_addr = (addr - 0x3F00) % 32;
                // Mirror sprite palettes
                if palette_addr == 0x10 || palette_addr == 0x14 || palette_addr == 0x18 || palette_addr == 0x1C {
                    palette_addr -= 0x10;
                }
                self.palette_table[palette_addr as usize] = value;
            }
            _ => panic!("Unexpected PPU write to address {:#X}", addr),
        }
        self.increment_vram_addr();
    }
    
    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            // Combine the VRAM and its mirror into one case.
            // We can just subtract 0x1000 from any address in the mirror range.
            0x2000..=0x3EFF => {
                let result = self.internal_data_buf;
                let mirrored_addr = addr & 0x2FFF; // This masks the address down to the 2000-2FFF range
                self.internal_data_buf = self.vram[self.mirror_vram_addr(mirrored_addr) as usize];
                result
            }
            0x3F00..=0x3FFF => {
                let mut palette_addr = (addr - 0x3F00) % 32;
                if palette_addr == 0x10 || palette_addr == 0x14 || palette_addr == 0x18 || palette_addr == 0x1C {
                    palette_addr -= 0x10;
                }
                self.palette_table[palette_addr as usize]
            }
        _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }
    pub fn peek_status(&self) -> u8 {
        let mut status = 0;
        // CORRECT WAY to check a bitflag
        if self.status.contains(StatusRegister::VBLANK_STARTED) {
            status |= 0b1000_0000;
        }
        // You could also add other flags here if needed for peeking, for example:
        // if self.status.contains(StatusRegister::SPRITE_0_HIT) {
        //     status |= 0b0100_0000;
        // }
        status
    }
}