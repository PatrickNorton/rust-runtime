use std::string::String;
use std::vec::Vec;

pub struct BaseFunction {
    name: String,
    local_count: u16,
    bytes: Vec<u8>,
}

impl BaseFunction {
    fn new(name: String, local_count: u16, bytes: Vec<u8>) -> BaseFunction {
        BaseFunction {
            name, local_count, bytes
        }
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_local_count(&self) -> u16 {
        self.local_count
    }

    fn get_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }
}
