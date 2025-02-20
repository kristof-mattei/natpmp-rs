use zerocopy::{Immutable, IntoBytes};

use super::Request;
use crate::VERSION;
use crate::protocol::MappingProtocol;
use crate::responses::MappingResponse;

#[derive(IntoBytes, Immutable)]
#[repr(C)]
pub(crate) struct UnmapAllPortsRequest {
    version: u8,
    protocol: MappingProtocol,
    _spacer: u16,
    internal_port: u16,
    external_port: u16,
    lifetime: u32,
}

impl UnmapAllPortsRequest {
    pub(crate) fn new(protocol: MappingProtocol) -> Self {
        Self {
            version: VERSION,
            protocol,
            _spacer: 0,
            // internal port, set to zero to remove all from this protocol
            internal_port: 0,
            // external port, set to zero as per spec
            external_port: 0,
            // lifetime, set to zero as per spec
            lifetime: 0,
        }
    }
}

impl Request for UnmapAllPortsRequest {
    type Response = MappingResponse;

    fn get_opcode(&self) -> u8 {
        self.protocol.into()
    }
}
