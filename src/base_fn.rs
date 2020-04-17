use crate::constant_loaders::load_std_str;
use crate::int_tools::bytes_index;
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
        let name = load_std_str(data, index);
        let var_count = bytes_index::<u16>(data, index);
        let fn_size = bytes_index::<u32>(data, index) as usize;
        let values = data[*index..*index + fn_size].to_vec();
        *index += fn_size;
        BaseFunction::new(name, var_count, values)
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
