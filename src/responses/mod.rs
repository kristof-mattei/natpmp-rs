use std::net::Ipv4Addr;
use std::num::NonZeroU16;

use bytes::{Buf, BytesMut};

use crate::errors::{NATPMPError, NATPMPResultError};
use crate::protocol::MappingProtocol;
use crate::requests::Request;
use crate::VERSION;

pub(crate) trait Response {
    const SIZE: usize;

    fn get_buffer() -> BytesMut {
        BytesMut::zeroed(Self::SIZE)
    }

    fn try_from_bytes(
        opcode: u8,
        buffer: &[u8],
        // buffer: &[u8],
    ) -> Result<Self, NATPMPError>
    where
        Self: std::marker::Sized;
}

pub(crate) fn parse_raw_response<R: Request>(
    request: &R,
    mut buffer: &[u8],
) -> Result<R::Response, NATPMPError> {
    let version = buffer.get_u8();

    if version != VERSION {
        return Err(NATPMPError::Response(NATPMPResultError::UnsupportedVersion));
    }

    let opcode = buffer.get_u8();

    // normally opcodes are supposed to be 128, 129 or 130, but 0 is allowed because of a bug in the RFC:
    // Source: https://www.rfc-editor.org/errata/rfc6886

    // note that this only happens when we send data with an unsupported version (which this code doesn't)
    // AND from testing, my UniFi doesn't return anything when I send VERSION = 1/2/3/4/5, OPCODE = 0

    // I'm not sure about the order of tests here, whether to test the validity of the opcode here
    // or rely on the result code
    // if !(128..=130).contains(&opcode) {
    //     return Err(NATPMPError::Response(NATPMPResultError::UnsupportedOpcode));
    // }

    if opcode & 0x7f != request.get_opcode() {
        // if we hit this the response received did match the request sent
        todo!()
    }

    let result_code = buffer.get_u16();

    if result_code != 0 {
        // this is bad, we need to still check the internal port to see if it matches the request
        // we can't just assume the response we got here is tied to the request we sent
        return Err(NATPMPResultError::try_from(result_code)
            .map_or_else(NATPMPError::Deserialize, NATPMPError::Response));
    }

    R::Response::try_from_bytes(opcode, buffer)
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MappingResponse {
    protocol: MappingProtocol,
    internal_port: NonZeroU16,
    external_port: u16,
    lifetime: u32,
    seconds_since_epoch: u32,
}

impl std::fmt::Display for MappingResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Protocol: {}, internal port: {}, external port {}, lifetime: {}, secondssince epoch: {}",
            self.protocol,
            self.internal_port,
            self.external_port,
            self.lifetime,
            self.seconds_since_epoch,
        )
    }
}

impl Response for MappingResponse {
    const SIZE: usize = 16;

    fn try_from_bytes(opcode: u8, mut buffer: &[u8]) -> Result<Self, NATPMPError> {
        // parse protocol
        let protocol: MappingProtocol = (opcode & 0x7f)
            .try_into()
            .map_err(NATPMPError::Deserialize)?;

        let seconds_since_epoch = buffer.get_u32();
        let internal_port = buffer.get_u16();
        let external_port = buffer.get_u16();
        let lifetime = buffer.get_u32();

        Ok(MappingResponse {
            protocol,
            seconds_since_epoch,
            internal_port: internal_port.try_into().unwrap(),
            external_port,
            lifetime,
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ExternalAddressResponse {
    seconds_since_epoch: u32,
    pub ipv4_address: Ipv4Addr,
}

impl Response for ExternalAddressResponse {
    const SIZE: usize = 12;

    fn try_from_bytes(_opcode: u8, mut buffer: &[u8]) -> Result<Self, NATPMPError> {
        let seconds_since_epoch = buffer.get_u32();
        let ip_address = buffer.get_u32();

        Ok(ExternalAddressResponse {
            seconds_since_epoch,
            ipv4_address: Ipv4Addr::from(ip_address),
        })
    }
}
