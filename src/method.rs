use std::boxed::Box;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::vec::Vec;

use crate::runtime::Runtime;
use crate::variable::Variable;

#[derive(Copy, Clone)]
pub enum InnerMethod<T> {
    Standard(u32),
    Native(fn(&T, Vec<Variable>, &mut Runtime)),
}

pub trait MethodClone {
    fn clone_box(&self) -> Box<dyn Method>;
}

pub trait Method: MethodClone {
    fn call(&self, args: (Vec<Variable>, &mut Runtime));
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
    fn hash<H: Hasher>(&self, state: &mut H) {
        unimplemented!()
    }
}

#[derive(Clone)]
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

impl<T: 'static> Method for StdMethod<T>
where
    T: Clone + Into<Variable>,
{
    fn call(&self, mut args: (Vec<Variable>, &mut Runtime)) {
        match &self.method {
            InnerMethod::Standard(index) => {
                let runtime = args.1; // FIXME: Insert type as argument
                let var: Variable = self.value.clone().into();
                args.0.insert(0, Variable::Type(var.get_type()));
                args.0.insert(0, var);
                runtime.push_stack(0, *index as u16, args.0, 0);
            }
            InnerMethod::Native(func) => {
                args.1.push_native();
                func(&self.value, args.0, args.1);
                args.1.pop_stack();
            }
        }
    }
}
