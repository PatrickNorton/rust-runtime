use std::collections::HashMap;

use crate::variable::Variable;
use crate::std_type::Type;
use crate::runtime::Runtime;
use crate::operator::Operator;

pub enum StdMethod {
    Standard(i32),
    Native(fn(StdVariable, Vec<Variable>, &Runtime)),
}

pub enum Name {
    Attribute(String),
    Operator(Operator),
}

pub struct StdVariable {
    cls: &'static Type,
    values: HashMap<Name, Variable>
}

impl StdVariable {
    pub fn str(&mut self, _runtime: &Runtime) -> String {
        unimplemented!();
    }
}
