use crate::custom_var::CustomVar;
use crate::executor;
use crate::looping::{IterResult, NativeIterator};
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::stack_frame::StackFrame;
use crate::std_type::Type;
use crate::variable::{FnResult, Variable};
use std::cell::Cell;
use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;

pub struct Generator {
    frame: Cell<Option<StackFrame>>,
    stack: Cell<Vec<Variable>>,
}

impl Generator {
    pub fn new(frame: StackFrame, stack: Vec<Variable>) -> Generator {
        Generator {
            frame: Cell::new(Option::Some(frame)),
            stack: Cell::new(stack),
        }
    }

    pub fn replace_vars(&self, frame: StackFrame, stack: Vec<Variable>) {
        assert!(self.frame.take().is_none());
        self.frame.replace(Option::Some(frame));
        self.stack.replace(stack);
    }

    pub fn take_frame(&self) -> Option<StackFrame> {
        self.frame.take()
    }

    pub fn take_stack(&self) -> Vec<Variable> {
        self.stack.take()
    }

    pub fn create(_args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        panic!(
            "Should not be creating generators\n{}",
            runtime.frame_strings()
        )
    }

    fn gen_type() -> Type {
        custom_class!(Generator, create, "Generator")
    }

    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push_native();
        runtime.add_generator(self)?;
        let result = executor::execute(runtime);
        runtime.pop_native();
        result
    }

    fn ret_self(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.into())
    }
}

impl CustomVar for Generator {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::gen_type()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        match op {
            Operator::Iter => StdMethod::new_native(self, Self::ret_self).into(),
            _ => unimplemented!("Generator.{}", op.name()),
        }
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        match name {
            "next" => StdMethod::new_native(self, Self::next_fn).into(),
            _ => unimplemented!("Generator.{}", name),
        }
    }
}

impl NativeIterator for Generator {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        runtime.add_generator(self)?;
        match executor::execute(runtime) {
            FnResult::Ok(_) => match runtime.pop_return() {
                Variable::Option(var) => IterResult::Ok(var.into()),
                _ => panic!("Expected option to be returned from generator"),
            },
            FnResult::Err(_) => IterResult::Err(()),
        }
    }
}

impl Debug for Generator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let frame = self.frame.take();
        let stack = self.stack.take();
        let result = f
            .debug_struct("Generator")
            .field("frame", &frame)
            .field("stack", &stack)
            .finish();
        self.frame.replace(frame);
        self.stack.replace(stack);
        result
    }
}
