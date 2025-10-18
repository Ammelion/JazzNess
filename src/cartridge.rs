#[derive(Debug, PartialEq)]
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
        // Check for the "NES" header tag
        if &raw[0..4] != NES_TAG {
            return Err("File is not in iNES file format".to_string());
        }

        // Determine the mapper number
        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        // Check iNES version
        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        // Determine screen mirroring type from control byte 6
        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOURSCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        // Calculate PRG and CHR ROM sizes
        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        // Check if a 512-byte trainer is present
        let skip_trainer = raw[6] & 0b100 != 0;

        // Determine the starting points for PRG and CHR ROM data
        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        // Extract the ROM data into vectors
        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper,
            screen_mirroring,
        })
    }
}