use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]

pub struct GameGenieCode {
    pub address: u16,
    pub new_data: u8,
    pub compare_data: Option<u8>,
}

unsafe impl Send for GameGenieCode {}
unsafe impl Sync for GameGenieCode {}

fn decode_gg_char(c: char) -> Option<u8> {
    match c {
        'A' => Some(0x0), 'P' => Some(0x1), 'Z' => Some(0x2), 'L' => Some(0x3),
        'G' => Some(0x4), 'I' => Some(0x5), 'T' => Some(0x6), 'Y' => Some(0x7),
        'E' => Some(0x8), 'O' => Some(0x9), 'X' => Some(0xA), 'U' => Some(0xB),
        'K' => Some(0xC), 'S' => Some(0xD), 'V' => Some(0xE), 'N' => Some(0xF),
        _ => None,
    }
}

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

    let address = 0x8000
        | (((n[3] & 7) as u16) << 12)
        | (((n[5] & 7) as u16) << 8)  | (((n[4] & 8) as u16) << 8)
        | (((n[2] & 7) as u16) << 4)  | (((n[1] & 8) as u16) << 4)
        |  ((n[4] & 7) as u16)        |  ((n[3] & 8) as u16);

    let new_data: u8;
    let compare_data: Option<u8>;

    if len == 8 {
        new_data =
              (((n[1] & 7) as u8) << 4) | (((n[0] & 8) as u8) << 4)
            |  ((n[0] & 7) as u8)        |  ((n[7] & 8) as u8);

        compare_data = Some(
              (((n[7] & 7) as u8) << 4) | (((n[6] & 8) as u8) << 4)
            |  ((n[6] & 7) as u8)        |  ((n[5] & 8) as u8)
        );
    } else {
         new_data =
              (((n[1] & 7) as u8) << 4) | (((n[0] & 8) as u8) << 4)
            |  ((n[0] & 7) as u8)        |  ((n[5] & 8) as u8);
        compare_data = None;
    }

    Ok(GameGenieCode {
        address,
        new_data,
        compare_data,
    })
}

