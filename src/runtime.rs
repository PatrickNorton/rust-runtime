use std::rc::Rc;
use std::vec::Vec;

use crate::executor;
use crate::file_info::FileInfo;
use crate::operator::Operator;
use crate::stack_frame::StackFrame;
use crate::variable::Variable;
use std::collections::HashMap;

pub struct Runtime {
    variables: Vec<Variable>,
    frames: Vec<StackFrame>,
    files: Vec<Rc<FileInfo>>,
    exception_frames: HashMap<Variable, Vec<(u32, u32)>>,
    exception_stack: Vec<Variable>,
}

impl Runtime {
    pub fn push(&mut self, var: Variable) {
        self.variables.push(var)
    }

    pub fn pop(&mut self) -> Variable {
        self.variables.pop().unwrap()
    }

    pub fn pop_bool(&mut self) -> bool {
        self.variables.pop().unwrap().to_bool(self)
    }

    pub fn top(&mut self) -> &Variable {
        self.variables.last().unwrap()
    }

    pub fn load_const(&self, index: u16) -> &Variable {
        &self.files.last().unwrap().get_constants()[index as usize]
    }

    pub fn load_value(&self, index: u16) -> &Variable {
        &self.frames.last().unwrap()[index as usize]
    }

    pub fn store_variable(&mut self, index: u16, value: Variable) {
        self.frames.last_mut().unwrap()[index as usize] = value;
    }

    pub fn call(&mut self, argc: u16) {}

    pub fn call_op(&mut self, var: Variable, o: Operator, args: Vec<Variable>) {
        unimplemented!()
    }

    pub fn goto(&mut self, pos: u32) {
        self.frames.last_mut().unwrap().jump(pos)
    }

    pub fn current_fn(&self) -> &Vec<u8> {
        self.files.last().unwrap().get_functions()
            [self.frames.last().unwrap().get_fn_number() as usize]
            .get_bytes()
    }

    pub fn current_pos(&self) -> usize {
        self.frames.last().unwrap().current_pos() as usize
    }

    pub fn advance(&mut self, pos: u32) {
        let jump = self.current_pos() as u32 + pos;
        self.goto(jump);
    }

    pub fn load_args(&mut self, argc: u16) -> Vec<Variable> {
        let mut args: Vec<Variable> = Vec::with_capacity(argc as usize);
        for i in 0..argc {
            args[(argc - i - 1) as usize] = self.pop()
        }
        return args;
    }

    pub fn push_stack(
        &mut self,
        var_count: u16,
        fn_no: u16,
        args: Vec<Variable>,
        info: Rc<FileInfo>,
    ) {
        let native = self.is_native();
        if Rc::ptr_eq(&info, self.files.last().unwrap()) {
            self.frames.push(StackFrame::new(var_count, fn_no, args));
        } else {
            self.frames
                .push(StackFrame::new_file(var_count, fn_no, args));
            self.files.push(info);
        }
        if native {
            executor::execute(self);
            assert!(self.is_native());
        }
    }

    pub fn pop_stack(&mut self) {
        for v in self.frames.last().unwrap().get_exceptions() {
            assert_eq!(
                self.exception_frames[v].last().unwrap().1 as usize,
                self.frames.len() - 1
            );
            self.exception_frames.get_mut(v).unwrap().pop();
            self.exception_stack.pop();
        }
        if self.frames.last().unwrap().is_new_file() {
            self.files.pop();
        }
        self.frames.pop();
    }

    pub fn is_native(&self) -> bool {
        self.frames.last().unwrap().is_native()
    }

    pub fn is_bottom_stack(&self) -> bool {
        self.frames.len() == 1
    }
}
