use crate::constant_loaders::load_std_str;
use crate::int_tools::bytes_index;
use std::string::String;
use std::vec::Vec;

#[derive(Debug)]
pub struct BaseFunction {
    name: String,
    local_count: u16,
    bytes: Vec<u8>,
    is_gen: bool,
}

impl BaseFunction {
    pub(crate) fn new(name: String, local_count: u16, bytes: Vec<u8>) -> BaseFunction {
        BaseFunction {
            name,
            local_count,
            bytes,
            is_gen: false,
        }
    }

    pub(crate) fn new_gen(name: String, local_count: u16, bytes: Vec<u8>) -> BaseFunction {
        BaseFunction {
            name,
            local_count,
            bytes,
            is_gen: true,
        }
    }

    pub fn parse(data: &[u8], index: &mut usize) -> BaseFunction {
        let name = load_std_str(data, index);
        let is_gen = data[*index] != 0;
        *index += 1;
        let var_count = bytes_index::<u16>(data, index);
        let fn_size = bytes_index::<u32>(data, index) as usize;
        let values = data[*index..*index + fn_size].to_vec();
        *index += fn_size;
        if is_gen {
            BaseFunction::new_gen(name, var_count, values)
        } else {
            BaseFunction::new(name, var_count, values)
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_local_count(&self) -> u16 {
        self.local_count
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn is_generator(&self) -> bool {
        self.is_gen
    }
}
