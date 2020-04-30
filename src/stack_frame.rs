use crate::variable::Variable;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Index, IndexMut};
use std::option::Option;
use std::rc::Rc;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    exception_handlers: HashSet<Variable>,
    variables: Vec<Variable>,
    function_number: u16,
    location: u32,
    native: bool,
    new_file: bool,
    parent: Option<Rc<RefCell<StackFrame>>>,
}

impl StackFrame {
    pub fn new(_var_count: u16, fn_no: u16, args: Vec<Variable>) -> StackFrame {
        StackFrame {
            exception_handlers: HashSet::new(),
            variables: args,
            function_number: fn_no,
            location: 0,
            native: false,
            new_file: false,
            parent: Option::None,
        }
    }

    pub fn new_file(_var_count: u16, fn_no: u16, args: Vec<Variable>) -> StackFrame {
        StackFrame {
            exception_handlers: HashSet::new(),
            variables: args,
            function_number: fn_no,
            location: 0,
            native: false,
            new_file: true,
            parent: Option::None,
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
            parent: Option::None,
        }
    }

    fn size(&self) -> usize {
        self.variables.len()
            + if self.parent.is_some() {
                (*self.parent.as_ref().unwrap()).borrow().size()
            } else {
                0
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
