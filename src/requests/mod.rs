use crate::responses::Response;

pub(crate) mod external_address_request;
pub(crate) mod mapping_request;
pub(crate) mod unmap_all_request;
pub(crate) mod unmap_request;

pub(crate) trait Request {
    type Response: Response;
    fn get_opcode(&self) -> u8;
}
