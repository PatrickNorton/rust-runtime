use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

use crate::variable::Variable;
use crate::base_fn::BaseFunction;

pub struct FileInfo {
    name: String,
    constants: Vec<Variable>,
    functions: Vec<BaseFunction>,
    exports: HashMap<String, u32>,
}

impl FileInfo {
    pub fn new(name: String, constants: Vec<Variable>,
               functions: Vec<BaseFunction>, exports: HashMap<String, u32>) -> FileInfo {
        FileInfo {
            name, constants, functions, exports
        }
    }

    pub fn get_constants(&self) -> &Vec<Variable> {
        &self.constants
    }

    pub fn get_functions(&self) -> &Vec<BaseFunction> {
        &self.functions
    }

    pub fn get_export(&self, name: &String) -> &Variable {
        &self.constants[self.exports[name] as usize]
    }

    pub fn execute() {
        unimplemented!();
    }
}
