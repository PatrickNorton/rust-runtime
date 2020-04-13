use std::boxed::Box;
use std::vec::Vec;

use crate::variable::Variable;
use crate::runtime::Runtime;

#[derive(Clone)]
pub enum InnerMethod<T> {
    Standard(i32),
    Native(fn(&T, &Vec<Variable>, &mut Runtime)),
}

pub trait MethodClone {
    fn clone_box(&self) -> Box<dyn Method>;
}

pub trait Method: MethodClone {
    fn call(&self, args: (&Vec<Variable>, &mut Runtime));
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

#[derive(Clone)]
pub struct StdMethod<T> where T: Clone {
    value: T,
    method: InnerMethod<T>,
}

impl<T> StdMethod<T> where T: Clone {
    pub(crate) fn new(value: T, method: InnerMethod<T>) -> StdMethod<T> {
        StdMethod { value, method }
    }
}

impl<T: 'static> Method for StdMethod<T> where T: Clone {
    fn call(&self, args: (&Vec<Variable>, &mut Runtime)) {
        match &self.method {
            InnerMethod::Standard(index) => unimplemented!(),
            InnerMethod::Native(func) => func(&self.value, args.0, args.1)
        }
    }
}
