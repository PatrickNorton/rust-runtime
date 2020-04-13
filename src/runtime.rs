use std::rc::Rc;
use std::vec::Vec;

use crate::variable::Variable;
use crate::stack_frame::StackFrame;
use crate::file_info::FileInfo;

pub struct Runtime {
    variables: Vec<Variable>,
    frames: Vec<Rc<StackFrame>>,
    files: Vec<Rc<FileInfo>>,
}

impl Runtime {
    pub fn push(&mut self, var: Variable) {
        self.variables.push(var)
    }

    pub fn pop(&mut self) -> Variable {
        self.variables.pop().unwrap()
    }

    pub fn call(&mut self) {

    }

    pub fn load_args(argc: u16) -> Vec<Variable> {
        let mut args: Vec<Variable> = Vec::with_capacity(argc as usize);
        for i in 0..argc {
            args[&argc - i - 1] = pop()
        }
        return args
    }

    // uint16_t varCount, uint16_t functionNumber, const std::vector<Variable>& args, FileInfo* info, FramePtr& frame
    pub fn push_stack(&mut self, var_count: u16, fn_no: u16, args: &mut Vec<Variable>, info: Rc<FileInfo>) {
        let native = self.isNative();
        if Rc::ptr_eq(&info, self.files.top()) {
            self.frames.push(StackFrame::new(var_count, fn_no, args));
        } else {
            self.frames.push(StackFrame::new_file(var_count, fn_no, args));
            self.files.push(info);
        }
        self.frames.top().load_args(args);
        if native {
            Executor::execute(self);
            assert!(self.isNative());
        }
    }
}
