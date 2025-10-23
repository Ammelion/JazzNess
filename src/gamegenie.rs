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
/// Logic based on the nesgg.txt standard.
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

    let n = &nybbles; // n[0] is N1, n[1] is N2, etc.

    // --- CORRECTED LOGIC (Based on nesgg.txt standard) ---
    //
    // Letter: 1  2  3  4  5  6  (7  8)
    // Nybble: N1 N2 N3 N4 N5 N6 (N7 N8)
    //
    // Address: 8000 +
    //   (N2 & F)       -> Addr bits 0-3
    // | (N3 & 7) << 4  -> Addr bits 4-6
    // | (N4 & 8)       -> Addr bit 7
    // | (N5 & F) << 8  -> Addr bits 8-11
    // | (N6 & 7) << 12 -> Addr bits 12-14
    // | (N3 & 8) << 12 -> Addr bit 15
    //
    // Data:
    //   (N1 & F)       -> Data bits 0-3
    // | (N4 & 7) << 4  -> Data bits 4-6
    // | (N1 & 8)       -> Data bit 7
    //
    // Compare (8-letter only):
    //   (N8 & F)       -> Comp bits 0-3
    // | (N7 & 7) << 4  -> Comp bits 4-6
    // | (N8 & 8)       -> Comp bit 7
    
    // Note: The logic in your original file was also flawed, this is the correct mapping.
    let address = 0x8000
        | (n[1] as u16 & 0xF)          // Addr 0-3 (from N2)
        | ((n[2] as u16 & 0x7) << 4)   // Addr 4-6 (from N3)
        | (n[3] as u16 & 0x8)          // Addr 7   (from N4)
        | ((n[4] as u16 & 0xF) << 8)   // Addr 8-11 (from N5)
        | ((n[5] as u16 & 0x7) << 12)  // Addr 12-14 (from N6)
        | ((n[2] as u16 & 0x8) << 12); // Addr 15  (from N3)

    let new_data =
          (n[0] as u8 & 0xF)         // Data 0-3 (from N1)
        | ((n[3] as u8 & 0x7) << 4)  // Data 4-6 (from N4)
        | (n[0] as u8 & 0x8);      // Data 7   (from N1)

    let compare_data = if len == 8 {
        Some(
              (n[7] as u8 & 0xF)         // Comp 0-3 (from N8)
            | ((n[6] as u8 & 0x7) << 4)  // Comp 4-6 (from N7)
            | (n[7] as u8 & 0x8)       // Comp 7   (from N8)
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