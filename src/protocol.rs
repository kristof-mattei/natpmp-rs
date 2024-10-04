use zerocopy::{Immutable, IntoBytes};

#[derive(Debug, Copy, Clone)]
#[allow(clippy::module_name_repetitions)]
#[derive(IntoBytes, Immutable)]
#[repr(u8)]
pub enum MappingProtocol {
    UDP = 1,
    TCP,
}

impl std::fmt::Display for MappingProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MappingProtocol::UDP => "UDP",
                MappingProtocol::TCP => "TCP",
            }
        )
    }
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
