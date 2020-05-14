use crate::custom_types::exceptions::stop_iteration;
use crate::custom_var::CustomVar;
use crate::name::Name;
use crate::runtime::Runtime;
use crate::std_variable::StdVariable;
use crate::variable::{FnResult, Variable};
use std::rc::Rc;

pub type IterResult = Result<Option<Variable>, ()>;

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
            .index(Name::Attribute("next".into()), runtime)?
            .call((Vec::new(), runtime));
        match result {
            FnResult::Ok(_) => Result::Ok(Option::Some(runtime.pop_return())),
            FnResult::Err(_) => {
                let error = runtime.pop();
                if error.get_type() == stop_iteration() {
                    Result::Ok(Option::None)
                } else {
                    runtime.push(error);
                    Result::Err(())
                }
            }
        }
    }
}
