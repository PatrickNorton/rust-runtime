use crate::base_fn::BaseFunction;
use crate::jump_table::JumpTable;
use crate::variable::Variable;
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

#[derive(Debug)]
pub struct FileInfo {
    name: String,
    constants: Vec<Variable>,
    functions: Vec<BaseFunction>,
    exports: HashMap<String, u32>,
    jump_tables: Vec<JumpTable>,
}

impl FileInfo {
    pub fn new(
        name: String,
        constants: Vec<Variable>,
        functions: Vec<BaseFunction>,
        exports: HashMap<String, u32>,
        jump_tables: Vec<JumpTable>,
    ) -> FileInfo {
        FileInfo {
            name,
            constants,
            functions,
            exports,
            jump_tables,
        }
    }

    pub fn temp() -> FileInfo {
        FileInfo {
            name: String::new(),
            constants: vec![],
            functions: vec![],
            exports: HashMap::new(),
            jump_tables: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_constants(&self) -> &[Variable] {
        &self.constants
    }

    pub fn get_functions(&self) -> &[BaseFunction] {
        &self.functions
    }

    pub fn get_export(&self, name: &str) -> &Variable {
        &self.constants[self.exports[name] as usize]
    }

    pub fn jump_table(&self, val: usize) -> &JumpTable {
        &self.jump_tables[val]
    }
}
