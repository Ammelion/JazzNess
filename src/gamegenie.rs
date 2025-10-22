// In src/gamegenie.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameGenieCode {
    pub address: u16,
    pub new_data: u8,
    pub compare_data: Option<u8>,
}

unsafe impl Send for GameGenieCode {}
unsafe impl Sync for GameGenieCode {}

/// Decodes a single Game Genie letter to its 4-bit nybble.
fn decode_gg_char(c: char) -> Option<u8> {
    match c {
        'A' => Some(0x0), 'P' => Some(0x1), 'Z' => Some(0x2), 'L' => Some(0x3),
        'G' => Some(0x4), 'I' => Some(0x5), 'T' => Some(0x6), 'Y' => Some(0x7),
        'E' => Some(0x8), 'O' => Some(0x9), 'X' => Some(0xA), 'U' => Some(0xB),
        'K' => Some(0xC), 'S' => Some(0xD), 'V' => Some(0xE), 'N' => Some(0xF),
        _ => None,
    }
}

/// Parses a 6 or 8-letter Game Genie code string into a struct.
///
/// Logic based on the NesDev wiki: https://www.nesdev.org/wiki/Game_Genie_codec
pub fn parse_game_genie_code(code: &str) -> Result<GameGenieCode, String> {
    let code = code.to_uppercase();
    let len = code.len();

    if len != 6 && len != 8 {
        return Err("Code must be 6 or 8 letters long.".to_string());
    }

    let nybbles: Vec<u8> = code.chars()
        .map(|c| decode_gg_char(c))
        .collect::<Option<Vec<u8>>>()
        .ok_or("Code contains invalid letters.".to_string())?;

    let n = &nybbles;

    // --- FINAL, CORRECTED LOGIC ---
    //
    // Letter: 1  2  3  4  5  6  (7  8)
    // Nybble: N1 N2 N3 N4 N5 N6 (N7 N8)
    //
    // Addr: 8000 +
    //   (N2 & F)       -> Addr bits 0-3
    // | (N3 & 7) << 4  -> Addr bits 4-6
    // | (N3 & 8) << 4  -> Addr bit 7
    // | (N5 & F) << 8  -> Addr bits 8-11
    // | (N6 & 7) << 12 -> Addr bits 12-14
    //
    // Data:
    //   (N1 & 7)       -> Data bits 0-2
    // | (N1 & 8)       -> Data bit 3
    // | (N4 & 7) << 4  -> Data bits 4-6
    // | (N4 & 8)       -> Data bit 7
    //
    // Compare (8-letter only):
    //   (N8 & 7)       -> Comp bits 0-2
    // | (N8 & 8)       -> Comp bit 3
    // | (N7 & 7) << 4  -> Comp bits 4-6
    // | (N7 & 8)       -> Comp bit 7

    let address = 0x8000
        | ((n[1] as u16 & 0xF))          // N2 bits 0-3 -> Addr 0-3
        | ((n[2] as u16 & 0x7) << 4)   // N3 bits 0-2 -> Addr 4-6
        | ((n[2] as u16 & 0x8) << 4)   // N3 bit 3    -> Addr 7
        | ((n[4] as u16 & 0xF) << 8)   // N5 bits 0-3 -> Addr 8-11
        | ((n[5] as u16 & 0x7) << 12); // N6 bits 0-2 -> Addr 12-14

    let new_data =
          ((n[0] as u8 & 0x7))         // N1 bits 0-2 -> Data 0-2
        | (n[0] as u8 & 0x8)         // N1 bit 3    -> Data 3  <--- FIXED
        | ((n[3] as u8 & 0x7) << 4)  // N4 bits 0-2 -> Data 4-6
        | ((n[3] as u8 & 0x8));      // N4 bit 3    -> Data 7

    let compare_data = if len == 8 {
        Some(
              ((n[7] as u8 & 0x7))         // N8 bits 0-2 -> Comp 0-2
            | (n[7] as u8 & 0x8)         // N8 bit 3    -> Comp 3  <--- FIXED
            | ((n[6] as u8 & 0x7) << 4)  // N7 bits 0-2 -> Comp 4-6
            | ((n[6] as u8 & 0x8))       // N7 bit 3    -> Comp 7
        )
    } else {
        None
    };

    Ok(GameGenieCode {
        address,
        new_data,
        compare_data,
    })
}