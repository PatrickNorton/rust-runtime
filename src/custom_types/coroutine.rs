use crate::custom_var::CustomVar;
use crate::executor;
use crate::looping::{IterResult, NativeIterator};
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::stack_frame::StackFrame;
use crate::std_type::Type;
use crate::variable::{FnResult, OptionVar, Variable};
use std::cell::Cell;
use std::fmt;
use std::fmt::{Debug, Formatter};
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
            runtime.stack_frames()
        )
    }

    fn gen_type() -> Type {
        custom_class!(Generator, create, "Generator")
    }

    fn next_fn(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push_native();
        runtime.add_generator(self.clone())?;
        let result = executor::execute(runtime);
        runtime.pop_native();
        result
    }

    fn ret_self(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.clone().into())
    }
}

impl CustomVar for Generator {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(op) => match op {
                Operator::Iter => StdMethod::new_native(self, Self::ret_self).into(),
                _ => unimplemented!("Generator.{}", name),
            },
            Name::Attribute(attr) => match attr.as_str() {
                "next" => StdMethod::new_native(self, Self::next_fn).into(),
                _ => unimplemented!("Generator.{}", attr),
            },
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::gen_type()
    }
}

impl NativeIterator for Generator {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        runtime.add_generator(self)?;
        match executor::execute(runtime) {
            FnResult::Ok(_) => match runtime.pop_return() {
                Variable::Option(i, val) => IterResult::Ok(OptionVar(i, val).into()),
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
