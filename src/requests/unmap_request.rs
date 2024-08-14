use std::num::NonZeroU16;

use zerocopy::AsBytes;

use super::Request;
use crate::protocol::MappingProtocol;
use crate::responses::MappingResponse;
use crate::VERSION;

#[derive(AsBytes)]
#[repr(C)]
pub(crate) struct UnmapPortRequest {
    version: u8,
    protocol: MappingProtocol,
    _spacer: u16,
    internal_port: NonZeroU16,
    external_port: u16,
    lifetime: u32,
}

impl UnmapPortRequest {
    pub(crate) fn new(protocol: MappingProtocol, private_port: NonZeroU16) -> Self {
        Self {
            version: VERSION,
            protocol,
            _spacer: 0,
            internal_port: private_port,
            // external port, set to zero as per spec
            external_port: 0,
            // lifetime, set to zero as per spec
            lifetime: 0,
        }
    }
}

impl Request for UnmapPortRequest {
    type Response = MappingResponse;

    fn get_opcode(&self) -> u8 {
        self.protocol.into()
    }
}
