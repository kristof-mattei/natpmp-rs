use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
// class NATPMPError(Exception):
//     """Generic exception state.  May be used to represent unknown errors."""
//     pass
pub enum NATPMPError {
    #[error("NAT Gateway error response as per RFC-6886")]
    Response(NATPMPResultError),
    #[error("Network error while trying to communiate with NAT Gateway")]
    Network(#[from] io::Error),
    #[error("NAT Gateway does not support NAT-PMP (inferred when calls fail after x retries)")]
    Unsupported,
    #[error("NAT Gateway responded with non-sensical response")]
    Deserialize(String),
    #[error("Generic error that doesn't fit in anything else")]
    Generic(String),
}

#[derive(Debug)]
pub enum NATPMPResultError {
    UnsupportedVersion = 1,
    NotAuthorizedRefused,
    NetworkFailure,
    OutOfResources,
    UnsupportedOpcode,
}

impl TryFrom<u16> for NATPMPResultError {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NATPMPResultError::UnsupportedVersion),
            2 => Ok(NATPMPResultError::NotAuthorizedRefused),
            3 => Ok(NATPMPResultError::NetworkFailure),
            4 => Ok(NATPMPResultError::OutOfResources),
            5 => Ok(NATPMPResultError::UnsupportedOpcode),
            _ => Err(String::from("Unrecognized Error Result Code")),
        }
    }
}
