use std::boxed::Box;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::vec::Vec;

use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub enum InnerMethod<T> {
    Standard(usize, u32),
    Native(fn(&T, Vec<Variable>, &mut Runtime) -> FnResult),
}

pub trait MethodClone {
    fn clone_box(&self) -> Box<dyn Method>;
}

pub trait Method: MethodClone + Debug {
    fn call(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult;
}

impl<T> MethodClone for T
where
    T: 'static + Method + Clone,
{
    fn clone_box(&self) -> Box<dyn Method> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Method> {
    fn clone(&self) -> Box<dyn Method> {
        self.clone_box()
    }
}

impl PartialEq for Box<dyn Method> {
    fn eq(&self, other: &Box<dyn Method>) -> bool {
        let left: *const dyn Method = self.as_ref();
        let right: *const dyn Method = other.as_ref();
        left == right
    }
}

impl Eq for Box<dyn Method> {}

impl Hash for Box<dyn Method> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct StdMethod<T>
where
    T: Clone + Into<Variable>,
{
    value: T,
    method: InnerMethod<T>,
}

impl<T> StdMethod<T>
where
    T: Clone + Into<Variable>,
{
    pub(crate) fn new(value: T, method: InnerMethod<T>) -> StdMethod<T> {
        StdMethod { value, method }
    }
}

impl<T: 'static + Debug> Method for StdMethod<T>
where
    T: Clone + Into<Variable>,
{
    fn call(&self, mut args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match &self.method {
            InnerMethod::Standard(file, index) => {
                let runtime = args.1; // FIXME: Insert type as argument
                let var: Variable = self.value.clone().into();
                args.0.insert(0, Variable::Type(var.get_type()));
                args.0.insert(0, var);
                runtime.push_stack(0, *index as u16, args.0, *file)?;
                FnResult::Ok(())
            }
            InnerMethod::Native(func) => {
                args.1.push_native();
                let result = func(&self.value, args.0, args.1);
                args.1.pop_native();
                result
            }
        }
    }
}

impl<T> Debug for InnerMethod<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerMethod::Standard(i, j) => f.debug_tuple("Standard").field(i).field(j).finish(),
            InnerMethod::Native(fn_) => f
                .debug_tuple("Native")
                .field(&format!("fn@{}", *fn_ as usize))
                .finish(),
        }
    }
}
