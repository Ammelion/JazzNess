pub mod frame;
use crate::ppu::NesPPU;
use frame::Frame;
use crate::palette;

fn sprite_palette(ppu: &NesPPU, palette_idx: u8) -> [u8; 4] {
    let start = 0x11 + (palette_idx * 4) as usize;
    [
        0,
        ppu.palette_table[start],
        ppu.palette_table[start + 1],
        ppu.palette_table[start + 2],
    ]
}

fn bg_palette(ppu: &NesPPU, tile_column: usize, tile_row: usize) -> [u8; 4] {
    let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = ppu.vram[0x3C0 + attr_table_idx];

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

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    // --- Draw Background ---
    let bank = ppu.ctrl.background_pattern_addr();

    // Iterate through the 960 tiles of the nametable
    for i in 0..0x3C0 {
        let tile_idx = ppu.vram[i] as u16;
        let tile_column = i % 32;
        let tile_row = i / 32;
        let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];
        let palette = bg_palette(ppu, tile_column, tile_row);

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                // Combine bits to get the 2-bit color value
                let value = (1 & lower) << 1 | (1 & upper);
                upper >>= 1;
                lower >>= 1;

                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[ppu.palette_table[0] as usize],
                    1 => palette::SYSTEM_PALLETE[palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[palette[3] as usize],
                    _ => unreachable!(),
                };
                frame.set_pixel(tile_column * 8 + x, tile_row * 8 + y, rgb);
            }
        }
    }
    if ppu.mask.contains(crate::ppu::MaskRegister::SHOW_SPRITES) {
        for i in (0..ppu.oam_data.len()).step_by(4).rev() {
            let tile_y = ppu.oam_data[i] as usize;
            let tile_idx = ppu.oam_data[i + 1] as u16;
            let attributes = ppu.oam_data[i + 2];
            let tile_x = ppu.oam_data[i + 3] as usize;

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

                    if value == 0 {
                        continue 'pixel_loop;
                    }
                    
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
                    
                    frame.set_pixel(pixel_x, pixel_y, rgb);
                }
            }
        }
    }
}