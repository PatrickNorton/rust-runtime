use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::{StdType, Type};
use crate::variable::{Name, Variable};

pub enum StdMethod {
    Standard(i32),
    Native(fn(StdVariable, Vec<Variable>, &mut Runtime)),
}

#[derive(Clone, Eq)]
pub struct StdVariable {
    value: Rc<RefCell<InnerVar>>,
}

#[derive(Clone, PartialEq, Eq)]
struct InnerVar {
    pub uuid: i128,
    pub cls: &'static StdType,
    pub values: HashMap<Name, Variable>,
}

impl StdVariable {
    pub fn str(&mut self, runtime: &mut Runtime) -> String {
        self.call_operator(Operator::Str, runtime);
        return runtime.pop().str(runtime);
    }

    pub fn bool(&mut self, runtime: &mut Runtime) -> bool {
        self.call_operator(Operator::Bool, runtime);
        runtime.pop().to_bool(runtime)
    }

    pub fn call_operator(&mut self, op: Operator, runtime: &mut Runtime) {
        unimplemented!()
    }

    pub fn call(&self, args: (&Vec<Variable>, &mut Runtime)) {
        self.value.borrow_mut().values[&Name::Operator(Operator::Call)].call(args)
    }

    pub fn index(&self, index: Name) -> Variable {
        self.value.borrow().values[&index].clone()
    }

    pub fn set(&self, index: String, value: Variable) {
        self.value
            .borrow_mut()
            .values
            .insert(Name::Attribute(index), value);
    }

    pub fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }

    pub fn get_type(&self) -> Type {
        Type::Standard(self.value.borrow_mut().cls)
    }
}

impl InnerVar {}

impl Hash for StdVariable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i128(self.value.borrow().uuid)
    }
}

impl PartialEq for StdVariable {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}
