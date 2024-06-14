#[derive(Debug, Copy, Clone)]
#[allow(clippy::module_name_repetitions)]
pub enum MappingProtocol {
    UDP = 1,
    TCP,
}

impl TryFrom<u8> for MappingProtocol {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(MappingProtocol::UDP),
            2 => Ok(MappingProtocol::TCP),
            _ => Err(format!("Invalid protocol code specified: {}", value)),
        }
    }
}

impl From<MappingProtocol> for u8 {
    fn from(value: MappingProtocol) -> Self {
        match value {
            MappingProtocol::TCP => 1,
            MappingProtocol::UDP => 2,
        }
    }
}
