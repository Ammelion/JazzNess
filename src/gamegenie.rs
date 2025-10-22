#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameGenieCode {
    pub address: u16,
    pub new_data: u8,
    pub compare_data: Option<u8>,
}