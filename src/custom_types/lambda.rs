use crate::custom_var::CustomVar;
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::stack_frame::StackFrame;
use crate::std_type::Type;
use crate::variable::{FnResult, Variable};
use std::rc::Rc;

#[derive(Debug)]
pub struct Lambda {
    file_no: usize,
    fn_no: u32,
    frame: StackFrame,
}

impl Lambda {
    pub fn new(file_no: usize, fn_no: u32, frame: StackFrame) -> Lambda {
        Lambda {
            file_no,
            fn_no,
            frame,
        }
    }

    fn take_frame(self: Rc<Self>) -> StackFrame {
        // Most of the time, the lambda will only have one referrer, so don't waste a clone
        match Rc::try_unwrap(self) {
            Result::Ok(lambda) => lambda.frame,
            Result::Err(lambda) => lambda.frame.clone(),
        }
    }

    fn call_now(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.call_now_with_frame(0, self.fn_no as u16, args, self.file_no, self.take_frame())
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!("Lambda objects should only be created through Bytecode::MakeFunction")
    }

    pub fn lambda_type() -> Type {
        custom_class!(Lambda, create, "lambda")
    }
}

impl CustomVar for Lambda {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::lambda_type()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        let func = match op {
            Operator::Call => Self::call_now,
            _ => unimplemented!(),
        };
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, _name: &str) -> Variable {
        unimplemented!()
    }

    fn call(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.call_now(args, runtime)
    }

    fn call_or_goto(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.push_stack_with_frame(0, self.fn_no as u16, args, self.file_no, self.take_frame());
        FnResult::Ok(())
    }
}
