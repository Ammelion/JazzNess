// ADD ALL THESE IMPORTS AT THE TOP
pub mod frame;
use crate::cartridge::Mirroring;
use crate::palette;
use crate::ppu::NesPPU;
use frame::Frame;

// HELPER FUNCTION FOR BACKGROUND PALETTES
fn bg_palette(ppu: &NesPPU, attribute_table: &[u8], tile_column: usize, tile_row: usize) -> [u8; 4] {
    let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = attribute_table[attr_table_idx];

    let palette_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        (0, 0) => attr_byte & 0b11,
        (1, 0) => (attr_byte >> 2) & 0b11,
        (0, 1) => (attr_byte >> 4) & 0b11,
        (1, 1) => (attr_byte >> 6) & 0b11,
        _ => panic!("should not happen"),
    };

    let palette_start: usize = 1 + (palette_idx as usize) * 4;
    [
        ppu.palette_table[0],
        ppu.palette_table[palette_start],
        ppu.palette_table[palette_start + 1],
        ppu.palette_table[palette_start + 2],
    ]
}

// HELPER FUNCTION FOR SPRITE PALETTES
fn sprite_palette(ppu: &NesPPU, palette_idx: u8) -> [u8; 4] {
    let start = 0x11 + (palette_idx * 4) as usize;
    [
        ppu.palette_table[0], // transparent
        ppu.palette_table[start],
        ppu.palette_table[start + 1],
        ppu.palette_table[start + 2],
    ]
}

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let scroll_x = ppu.scroll.scroll_x as i32;
    let scroll_y = ppu.scroll.scroll_y as i32;

    // --- Draw Background ---
    if ppu.mask.contains(crate::ppu::MaskRegister::SHOW_BACKGROUND) {
        let base_nametable_addr = ppu.ctrl.nametable_addr();
        let vram = &ppu.vram;

        for y in 0..240 {
            for x in 0..256 {
                let world_x = (x as i32 + scroll_x) as u32;
                let world_y = (y as i32 + scroll_y) as u32;

                let nametable_x = (world_x / 256) % 2;
                let nametable_y = (world_y / 240) % 2;

                let nametable_idx = match (base_nametable_addr, nametable_x, nametable_y) {
                    (0x2000, 0, 0) => 0, (0x2000, 1, 0) => 1, (0x2000, 0, 1) => 2, (0x2000, 1, 1) => 3,
                    (0x2400, 0, 0) => 1, (0x2400, 1, 0) => 0, (0x2400, 0, 1) => 3, (0x2400, 1, 1) => 2,
                    (0x2800, 0, 0) => 2, (0x2800, 1, 0) => 3, (0x2800, 0, 1) => 0, (0x2800, 1, 1) => 1,
                    (0x2C00, 0, 0) => 3, (0x2C00, 1, 0) => 2, (0x2C00, 0, 1) => 1, (0x2C00, 1, 1) => 0,
                    _ => unreachable!(),
                };

                let page_idx = match ppu.mirroring {
                    Mirroring::VERTICAL => [0, 1, 0, 1][nametable_idx],
                    Mirroring::HORIZONTAL => [0, 0, 1, 1][nametable_idx],
                    _ => nametable_idx,
                };
                let nametable_ptr = &vram[(page_idx * 0x400)..((page_idx + 1) * 0x400)];

                let tile_x = (world_x % 256) / 8;
                let tile_y = (world_y % 240) / 8;
                let tile_idx_in_nametable = tile_y * 32 + tile_x;

                let tile_id = nametable_ptr[tile_idx_in_nametable as usize] as u16;
                let bank = ppu.ctrl.background_pattern_addr();
                let tile = &ppu.chr_rom[(bank + tile_id * 16) as usize..];
                
                let palette = bg_palette(ppu, &nametable_ptr[0x3c0..0x400], tile_x as usize, tile_y as usize);

                let pixel_in_tile_x = world_x % 8;
                let pixel_in_tile_y = world_y % 8;
                
                let upper = tile[pixel_in_tile_y as usize];
                let lower = tile[(pixel_in_tile_y + 8) as usize];
                
                let value = ((lower >> (7 - pixel_in_tile_x)) & 1) << 1 | ((upper >> (7 - pixel_in_tile_x)) & 1);
                
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[ppu.palette_table[0] as usize],
                    1 => palette::SYSTEM_PALLETE[palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[palette[3] as usize],
                    _ => unreachable!(),
                };
                frame.set_pixel(x as usize, y as usize, rgb);
            }
        }
    }

    // --- Draw Sprites ---
    if ppu.mask.contains(crate::ppu::MaskRegister::SHOW_SPRITES) {
        for i in (0..ppu.oam_data.len()).step_by(4).rev() {
            let tile_y = ppu.oam_data[i] as usize;
            let tile_idx = ppu.oam_data[i + 1] as u16;
            let attributes = ppu.oam_data[i + 2];
            let tile_x = ppu.oam_data[i + 3] as usize;

            if tile_y >= 239 {
                continue;
            }

            let flip_vertical = (attributes >> 7) & 1 == 1;
            let flip_horizontal = (attributes >> 6) & 1 == 1;
            let palette_idx = attributes & 0b11;
            let sprite_palette = sprite_palette(ppu, palette_idx);
            let bank = ppu.ctrl.sprite_pattern_addr();
            let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];

            for y in 0..=7 {
                let mut upper = tile[y];
                let mut lower = tile[y + 8];

                'pixel_loop: for x in (0..=7).rev() {
                    let value = (1 & lower) << 1 | (1 & upper);
                    upper >>= 1;
                    lower >>= 1;

                    if value == 0 { continue 'pixel_loop; }

                    let rgb = match value {
                        1 => palette::SYSTEM_PALLETE[sprite_palette[1] as usize],
                        2 => palette::SYSTEM_PALLETE[sprite_palette[2] as usize],
                        3 => palette::SYSTEM_PALLETE[sprite_palette[3] as usize],
                        _ => unreachable!(),
                    };

                    let pixel_x = match flip_horizontal {
                        true => tile_x + 7 - x,
                        false => tile_x + x,
                    };
                    let pixel_y = match flip_vertical {
                        true => tile_y + 7 - y,
                        false => tile_y + y,
                    };
                    
                    if pixel_x < 256 && pixel_y < 240 {
                        frame.set_pixel(pixel_x, pixel_y, rgb);
                    }
                }
            }
        }
    }
}