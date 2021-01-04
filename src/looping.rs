use crate::custom_var::CustomVar;
use crate::name::Name;
use crate::runtime::Runtime;
use crate::std_variable::StdVariable;
use crate::variable::{FnResult, OptionVar, Variable};
use std::mem::take;
use std::rc::Rc;

pub type IterResult = Result<IterOk, ()>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IterOk {
    Normal(Option<Variable>),
    Vec(Option<Vec<Variable>>),
}

#[derive(Debug, Clone)]
pub enum Iterator {
    Native(Rc<dyn NativeIterator>),
    NonNative(StdVariable),
}

pub trait NativeIterator: CustomVar {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult;
}

impl Iterator {
    pub fn next(&self, runtime: &mut Runtime) -> IterResult {
        match self {
            Iterator::Native(val) => val.clone().next(runtime),
            Iterator::NonNative(val) => Self::next_non_native(val, runtime),
        }
    }

    fn next_non_native(val: &StdVariable, runtime: &mut Runtime) -> IterResult {
        let result = val
            .index(Name::Attribute("next"), runtime)?
            .call((Vec::new(), runtime));
        match result {
            FnResult::Ok(_) => match runtime.pop_return() {
                Variable::Option(i, o) => IterResult::Ok(OptionVar(i, o).into()),
                _ => panic!("Expected iterator to return an option"),
            },
            FnResult::Err(_) => IterResult::Err(()),
        }
    }
}

impl From<Rc<dyn NativeIterator>> for Iterator {
    fn from(x: Rc<dyn NativeIterator>) -> Self {
        Iterator::Native(x)
    }
}

impl From<Option<Variable>> for IterOk {
    fn from(x: Option<Variable>) -> Self {
        IterOk::Normal(x)
    }
}

impl From<Option<Vec<Variable>>> for IterOk {
    fn from(x: Option<Vec<Variable>>) -> Self {
        IterOk::Vec(x)
    }
}

impl From<OptionVar> for IterOk {
    fn from(x: OptionVar) -> Self {
        IterOk::Normal(x.into())
    }
}

impl IterOk {
    pub fn take_first(self) -> Option<Variable> {
        match self {
            IterOk::Normal(v) => v,
            IterOk::Vec(v) => v.map(|mut x| take(&mut x[0])),
        }
    }
}
