use std::collections::HashSet;
use std::option::Option;
use std::rc::Rc;
use std::vec::Vec;

use crate::variable::Variable;

pub struct StackFrame {
    exception_handlers: HashSet<Variable>,
    variables: Vec<Variable>,
    function_number: u16,
    location: u32,
    native: bool,
    new_file: bool,
    parent: Option<Rc<StackFrame>>,
}

impl StackFrame {
    fn size(&self) -> usize {
        self.variables.len() + if self.parent.is_some() { self.parent.as_ref().unwrap().size() } else { 0 }
    }

    pub fn current_pos(&self) -> u32 {
        self.location  // Ignore "cannot move" error here and other similar places
    }

    pub fn advance(&mut self, pos: u32) {
        self.location += pos;
    }

    pub fn jump(&mut self, pos: u32) {
        self.location = pos;
    }

    pub fn get_fn_number(&self) -> u16 {
        self.function_number
    }

    pub fn load_args(&mut self, args: &mut Vec<Variable>) {
        self.variables.append(args)
    }

    pub fn store(&mut self, pos: u32, var: Variable) {
        self.variables[pos as usize] = var;
    }

    pub fn add_exception_handler(&mut self, var: Variable) {
        self.exception_handlers.insert(var);
    }

    pub fn remove_exception_handler(&mut self, var: Variable) {
        self.exception_handlers.remove(&var);
    }

    pub fn get_exceptions(&self) -> &HashSet<Variable> {
        &self.exception_handlers
    }

    pub fn is_native(&self) -> bool {
        self.native
    }

    pub fn is_new_file(&self) -> bool {
        self.new_file
    }
}
