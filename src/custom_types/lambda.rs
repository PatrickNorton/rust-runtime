use crate::custom_var::CustomVar;
use crate::method::StdMethod;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::stack_frame::StackFrame;
use crate::std_type::Type;
use crate::variable::{FnResult, Name, Variable};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Lambda {
    file_no: usize,
    fn_no: u32,
    frame: Rc<RefCell<StackFrame>>,
}

impl Lambda {
    pub fn new(file_no: usize, fn_no: u32, frame: Rc<RefCell<StackFrame>>) -> Lambda {
        Lambda {
            file_no,
            fn_no,
            frame,
        }
    }

    fn get_op(self: &Rc<Self>, op: Operator) -> Variable {
        let func = match op {
            Operator::Call => Self::call_now,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn call_now(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.call_now_with_frame(0, self.fn_no as u16, args, self.file_no, self.frame.clone())
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!("Lambda objects should only be created through Bytecode::MakeFunction")
    }

    pub fn lambda_type() -> Type {
        custom_class!(Lambda, create, "lambda")
    }
}

impl CustomVar for Lambda {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(o) => self.get_op(o),
            Name::Attribute(_) => unimplemented!(),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::lambda_type()
    }

    fn call(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.call_now(args, runtime)
    }

    fn call_or_goto(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.push_stack_with_frame(0, self.fn_no as u16, args, self.file_no, self.frame.clone());
        FnResult::Ok(())
    }
}
