use std::string::String;
use std::vec::Vec;

pub struct BaseFunction {
    name: String,
    local_count: u16,
    bytes: Vec<u8>,
}

impl BaseFunction {
    pub(crate) fn new(name: String, local_count: u16, bytes: Vec<u8>) -> BaseFunction {
        BaseFunction {
            name,
            local_count,
            bytes,
        }
    }

    pub fn parse(data: &Vec<u8>, index: &mut usize) -> BaseFunction {
        unimplemented!()
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_local_count(&self) -> u16 {
        self.local_count
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }
}
