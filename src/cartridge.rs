#[derive(Debug, PartialEq, Clone)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOURSCREEN,
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384; // 16 KiB
const CHR_ROM_PAGE_SIZE: usize = 8192;  // 8 KiB

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        if &raw[0..4] != NES_TAG {
            return Err("File is not in iNES file format".to_string());
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOURSCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper,
            screen_mirroring,
        })
    }

    // ADD THE FOLLOWING METHODS
    
    pub fn read(&self, addr: u16) -> u8 {
        match self.mapper {
            0 => { // Mapper 0 (NROM)
                let mut mapped_addr = addr as usize;
                if addr >= 0x8000 {
                    mapped_addr -= 0x8000;
                    // Mirror PRG ROM if it's only 16KB
                    if self.prg_rom.len() == PRG_ROM_PAGE_SIZE {
                        mapped_addr %= PRG_ROM_PAGE_SIZE;
                    }
                }
                self.prg_rom[mapped_addr]
            },
            _ => {
                // For now, we'll just panic for unsupported mappers.
                // Later you could implement more complex mappers here.
                panic!("Mapper {} not supported yet", self.mapper);
            }
        }
    }

    // You will need this write method later for more complex mappers.
    // For Mapper 0, writes to the ROM are ignored.
    pub fn write(&mut self, _addr: u16, _data: u8) {
        match self.mapper {
            0 => { /* Mapper 0 is not writable */ },
            _ => panic!("Mapper {} not supported yet", self.mapper),
        }
    }
}