use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use num::BigInt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CustomVarWrapper {
    value: Rc<dyn CustomVar>,
}

pub trait CustomVar: Debug {
    fn get_attr(&self, name: Name) -> Variable;
    fn set(&mut self, name: Name, object: Variable);
    fn get_type(&self) -> Type;

    fn call(&mut self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.call_op(Operator::Call, args, runtime)
    }

    fn call_op(
        &mut self,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        self.get_attr(Name::Operator(operator))
            .call((args, runtime))
    }

    fn str(&mut self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_op(Operator::Str, vec![], runtime)?;
        runtime.pop().str(runtime)
    }

    fn int(&mut self, runtime: &mut Runtime) -> Result<BigInt, ()> {
        self.call_op(Operator::Int, vec![], runtime)?;
        runtime.pop().int(runtime)
    }

    fn bool(&mut self, runtime: &mut Runtime) -> Result<bool, ()> {
        self.call_op(Operator::Bool, vec![], runtime)?;
        runtime.pop().to_bool(runtime)
    }
}

impl CustomVarWrapper {
    pub fn new(value: Rc<dyn CustomVar>) -> CustomVarWrapper {
        CustomVarWrapper { value }
    }
}

impl Deref for CustomVarWrapper {
    type Target = Rc<dyn CustomVar>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Hash for CustomVarWrapper {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!()
    }
}
