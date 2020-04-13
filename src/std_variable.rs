use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Index};
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

use enum_map::EnumMap;

use crate::variable::{Variable, Name};
use crate::std_type::Type;
use crate::runtime::Runtime;
use crate::operator::Operator;

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
    pub cls: &'static Type,
    pub values: HashMap<String, Variable>,
    pub operators: EnumMap<Operator, Variable>
}

impl StdVariable {
    pub fn str(&mut self, runtime: &mut Runtime) -> String {
        self.call_operator(Operator::Str, runtime);
        return runtime.pop().str(runtime)
    }

    pub fn call_operator(&mut self, op: Operator, runtime: &mut Runtime) {
        unimplemented!()
    }

    pub fn call(&self, args: (&Vec<Variable>, &mut Runtime)) {
        self.value.borrow_mut().operators[Operator::Call].call(args)
    }

    pub fn index(&self, index: Name) -> Variable {
        return match index {
            Name::Operator(op) => self.value.borrow().operators[op].clone(),
            Name::Attribute(str) => self.value.borrow().values[&str].clone(),
        }
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
