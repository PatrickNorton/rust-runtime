use crate::runtime::Runtime;
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
    file_number: usize,
    location: u32,
    native: bool,
    stack_height: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SFInfo {
    function_number: u16,
    file_number: usize,
    current_pos: u32,
    native: bool,
}

impl StackFrame {
    pub fn new(
        var_count: u16,
        fn_no: u16,
        file_no: usize,
        mut args: Vec<Variable>,
        stack_height: usize,
    ) -> StackFrame {
        if let Option::Some(val) = var_count.checked_sub(args.len() as u16) {
            args.reserve(val as usize);
        }
        StackFrame {
            exception_handlers: HashSet::new(),
            variables: args,
            function_number: fn_no,
            file_number: file_no,
            location: 0,
            native: false,
            stack_height,
        }
    }

    pub fn native() -> StackFrame {
        StackFrame {
            exception_handlers: HashSet::new(),
            variables: vec![],
            function_number: 0,
            file_number: 0,
            location: 0,
            native: true,
            stack_height: 0,
        }
    }

    pub fn from_old(
        var_count: u16,
        fn_no: u16,
        file_no: usize,
        args: Vec<Variable>,
        mut parent: StackFrame,
        stack_height: usize,
    ) -> StackFrame {
        parent.variables.extend(args);
        if let Option::Some(val) = var_count.checked_sub(parent.variables.len() as u16) {
            parent.variables.reserve(val as usize);
        }
        StackFrame {
            exception_handlers: parent.exception_handlers,
            variables: parent.variables,
            function_number: fn_no,
            file_number: file_no,
            location: 0,
            native: false,
            stack_height,
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

    pub fn remove_exception_handler(&mut self, var: &Variable) {
        self.exception_handlers.remove(var);
    }

    pub fn get_exceptions(&self) -> &HashSet<Variable> {
        &self.exception_handlers
    }

    pub fn is_native(&self) -> bool {
        self.native
    }

    pub fn file_no(&self) -> usize {
        self.file_number
    }

    pub fn original_stack_height(&self) -> usize {
        self.stack_height
    }

    pub fn exc_info(&self) -> SFInfo {
        SFInfo::new(
            self.function_number,
            self.file_number,
            self.location,
            self.native,
        )
    }

    pub fn len(&self) -> usize {
        self.variables.len()
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
            self.variables.push(Variable::default())
        }
        &mut self.variables[index]
    }
}

impl SFInfo {
    pub fn new(function_number: u16, file_number: usize, current_pos: u32, native: bool) -> SFInfo {
        SFInfo {
            function_number,
            file_number,
            current_pos,
            native,
        }
    }

    pub fn fn_no(&self) -> u16 {
        self.function_number
    }

    pub fn file_no(&self) -> usize {
        self.file_number
    }

    pub fn is_native(&self) -> bool {
        self.native
    }

    pub fn current_pos(&self) -> u32 {
        self.current_pos
    }
}

pub fn frame_strings(frames: impl Iterator<Item = SFInfo>, runtime: &Runtime) -> String {
    let mut result = String::new();
    for frame in frames {
        if !frame.is_native() {
            let file = &runtime.file_no(frame.file_no());
            let fn_no = frame.fn_no();
            let fn_pos = frame.current_pos();
            let func = &file.get_functions()[fn_no as usize];
            let fn_name = func.get_name();
            result.push_str(&*format!(
                "    at {}:{} ({})\n",
                fn_name,
                fn_pos,
                file.get_name()
            ))
        } else {
            result.push_str("    at [unknown native function]\n")
        }
    }
    result
}
