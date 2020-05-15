use crate::variable::Variable;
use std::collections::HashSet;
use std::ops::{Index, IndexMut};
use std::option::Option;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    exception_handlers: HashSet<Variable>,
    variables: Vec<Variable>,
    function_number: u16,
    location: u32,
    native: bool,
    new_file: bool,
}

impl StackFrame {
    pub fn new(var_count: u16, fn_no: u16, mut args: Vec<Variable>) -> StackFrame {
        if let Option::Some(val) = var_count.checked_sub(args.len() as u16) {
            args.reserve(val as usize);
        }
        StackFrame {
            exception_handlers: HashSet::new(),
            variables: args,
            function_number: fn_no,
            location: 0,
            native: false,
            new_file: false,
        }
    }

    pub fn new_file(var_count: u16, fn_no: u16, mut args: Vec<Variable>) -> StackFrame {
        if let Option::Some(val) = var_count.checked_sub(args.len() as u16) {
            args.reserve(val as usize);
        }
        StackFrame {
            exception_handlers: HashSet::new(),
            variables: args,
            function_number: fn_no,
            location: 0,
            native: false,
            new_file: true,
        }
    }

    pub fn native() -> StackFrame {
        StackFrame {
            exception_handlers: HashSet::new(),
            variables: vec![],
            function_number: 0,
            location: 0,
            native: true,
            new_file: false,
        }
    }

    pub fn from_old(
        var_count: u16,
        fn_no: u16,
        args: Vec<Variable>,
        mut parent: StackFrame,
    ) -> StackFrame {
        parent.variables.extend(args);
        if let Option::Some(val) = var_count.checked_sub(parent.variables.len() as u16) {
            parent.variables.reserve(val as usize);
        }
        StackFrame {
            exception_handlers: parent.exception_handlers,
            variables: parent.variables,
            function_number: fn_no,
            location: 0,
            native: false,
            new_file: false,
        }
    }

    pub fn from_old_new_file(
        var_count: u16,
        fn_no: u16,
        args: Vec<Variable>,
        mut parent: StackFrame,
    ) -> StackFrame {
        parent.variables.extend(args);
        if let Option::Some(val) = var_count.checked_sub(parent.variables.len() as u16) {
            parent.variables.reserve(val as usize);
        }
        StackFrame {
            exception_handlers: parent.exception_handlers,
            variables: parent.variables,
            function_number: fn_no,
            location: 0,
            native: false,
            new_file: true,
        }
    }

    pub fn current_pos(&self) -> u32 {
        self.location // Ignore "cannot move" error here and other similar places
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

impl Index<usize> for StackFrame {
    type Output = Variable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.variables[index]
    }
}

impl IndexMut<usize> for StackFrame {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        while self.variables.len() <= index {
            self.variables.push(Variable::Null())
        }
        &mut self.variables[index]
    }
}
