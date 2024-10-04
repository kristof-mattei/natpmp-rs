use std::num::NonZeroU16;

use zerocopy::{Immutable, IntoBytes};

use super::Request;
use crate::protocol::MappingProtocol;
use crate::responses::MappingResponse;
use crate::VERSION;

#[derive(IntoBytes, Immutable)]
#[repr(C)]
pub(crate) struct MappingRequest {
    version: u8,
    protocol: MappingProtocol,
    _spacer: u16,
    internal_port: NonZeroU16,
    external_port: u16,
    lifetime: u32,
}

impl Request for MappingRequest {
    type Response = MappingResponse;

    fn get_opcode(&self) -> u8 {
        self.protocol.into()
    }
}

impl MappingRequest {
    pub(crate) fn new(
        protocol: MappingProtocol,
        private_port: NonZeroU16,
        public_port: u16,
        lifetime: u32,
    ) -> Self {
        Self {
            version: VERSION,
            protocol,
            _spacer: 0,
            internal_port: private_port,
            external_port: public_port,
            lifetime,
        }
    }
}
