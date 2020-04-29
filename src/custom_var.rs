use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use num::BigInt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct CustomVarWrapper {
    value: Box<dyn CustomVar>,
}

pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn CustomVar>;
}

impl<T> CloneBox for T
where
    T: 'static + CustomVar + Clone,
{
    fn clone_box(&self) -> Box<dyn CustomVar> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CustomVar> {
    fn clone(&self) -> Box<dyn CustomVar> {
        self.clone_box()
    }
}

pub trait CustomVar: Debug + CloneBox {
    fn get_attr(&self, name: Name) -> Variable;
    fn set(&self, name: Name, object: Variable);
    fn get_type(&self) -> Type;

    fn call(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.call_op(Operator::Call, args, runtime)
    }

    fn call_op(&self, operator: Operator, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.get_attr(Name::Operator(operator))
            .call((args, runtime))
    }

    fn str(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_op(Operator::Str, vec![], runtime)?;
        runtime.pop().str(runtime)
    }

    fn int(&self, runtime: &mut Runtime) -> Result<BigInt, ()> {
        self.call_op(Operator::Int, vec![], runtime)?;
        runtime.pop().int(runtime)
    }

    fn bool(&self, runtime: &mut Runtime) -> Result<bool, ()> {
        self.call_op(Operator::Bool, vec![], runtime)?;
        runtime.pop().to_bool(runtime)
    }
}

impl CustomVarWrapper {
    pub fn new(value: Box<dyn CustomVar>) -> CustomVarWrapper {
        CustomVarWrapper { value }
    }
}

impl Deref for CustomVarWrapper {
    type Target = Box<dyn CustomVar>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Hash for CustomVarWrapper {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!()
    }
}

impl<T: 'static> From<Box<T>> for Variable
where
    T: CustomVar,
{
    fn from(value: Box<T>) -> Self {
        Variable::Custom(CustomVarWrapper::new(value))
    }
}
