use std::num::NonZeroU16;

use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    protocol::MappingProtocol,
    responses::{ExternalAddressResponse, MappingResponse, Response},
    VERSION,
};

pub(crate) trait Request {
    type Response: Response;
    fn get_opcode(&self) -> u8;
    fn to_bytes(&self) -> Bytes;
}

impl Request for ExternalAddressRequest {
    type Response = ExternalAddressResponse;

    fn get_opcode(&self) -> u8 {
        0
    }

    fn to_bytes(&self) -> Bytes {
        Bytes::from_static(&[VERSION, 0])
    }
}

impl Request for MappingRequest {
    type Response = MappingResponse;

    fn get_opcode(&self) -> u8 {
        self.protocol.into()
    }

    fn to_bytes(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.put_u8(VERSION);
        bytes.put_u8(self.protocol.into());
        // spacer
        bytes.put_u16(0);
        bytes.put_u16((self.internal_port).into());
        bytes.put_u16(self.external_port);
        bytes.put_u32(self.lifetime);

        bytes.into()
    }
}
pub(crate) struct ExternalAddressRequest {}
pub(crate) struct MappingRequest {
    protocol: MappingProtocol,
    internal_port: NonZeroU16,
    external_port: u16,
    lifetime: u32,
}
impl MappingRequest {
    pub(crate) fn new(
        protocol: MappingProtocol,
        private_port: NonZeroU16,
        public_port: u16,
        lifetime: u32,
    ) -> Self {
        Self {
            protocol,
            internal_port: private_port,
            external_port: public_port,
            lifetime,
        }
    }
}

pub(crate) struct UnmapPortRequest {
    protocol: MappingProtocol,
    internal_port: NonZeroU16,
}

impl UnmapPortRequest {
    pub(crate) fn new(protocol: MappingProtocol, private_port: NonZeroU16) -> Self {
        Self {
            protocol,
            internal_port: private_port,
        }
    }
}

impl Request for UnmapPortRequest {
    type Response = MappingResponse;

    fn get_opcode(&self) -> u8 {
        todo!()
    }

    fn to_bytes(&self) -> Bytes {
        todo!()
    }
}

pub(crate) struct UnmapAllPortsRequest {
    protocol: MappingProtocol,
}
impl UnmapAllPortsRequest {
    pub(crate) fn new(protocol: MappingProtocol) -> Self {
        Self { protocol }
    }
}

impl Request for UnmapAllPortsRequest {
    type Response = MappingResponse;

    fn get_opcode(&self) -> u8 {
        todo!()
    }

    fn to_bytes(&self) -> Bytes {
        todo!()
    }
}
