use zerocopy::{Immutable, IntoBytes};

use super::Request;
use crate::responses::ExternalAddressResponse;
use crate::VERSION;

impl Request for ExternalAddressRequest {
    type Response = ExternalAddressResponse;

    fn get_opcode(&self) -> u8 {
        // or self.opcode
        0
    }
}

#[derive(IntoBytes, Immutable)]
#[repr(C)]
pub(crate) struct ExternalAddressRequest {
    version: u8,
    opcode: u8,
}

impl ExternalAddressRequest {
    pub(crate) fn new() -> Self {
        Self {
            version: VERSION,
            opcode: 0,
        }
    }
}
