use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};
use std::boxed::Box;
use std::cmp::{Eq, PartialEq};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::vec::Vec;

pub type NativeMethod<T> = fn(&T, Vec<Variable>, &mut Runtime) -> FnResult;
pub type NativeCopyMethod<T> = fn(T, Vec<Variable>, &mut Runtime) -> FnResult;

pub enum InnerMethod<T> {
    Standard(usize, u32),
    Native(NativeMethod<T>),
    Move(NativeCopyMethod<T>),
}

// Not derived b/c #[derive] only implements Clone/Copy when T is Clone/Copy, which is not a bound
// here (function pointers are always Copy)
impl<T> Clone for InnerMethod<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for InnerMethod<T> {}

impl<T> InnerMethod<T>
where
    T: Clone + Into<Variable>,
{
    pub fn call(self, callee: &T, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        match self {
            InnerMethod::Standard(file, index) => {
                let var: Variable = (*callee).clone().into();
                args.insert(0, Variable::Type(var.get_type()));
                args.insert(0, var);
                runtime.call_now(0, index as u16, args, file)
            }
            InnerMethod::Native(func) => runtime.call_native_method(func, callee, args),
            InnerMethod::Move(func) => runtime.call_copy_method(func, callee.clone(), args),
        }
    }

    pub fn call_or_goto(
        self,
        callee: &T,
        mut args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        match self {
            InnerMethod::Standard(file, index) => {
                let var: Variable = (*callee).clone().into();
                args.insert(0, Variable::Type(var.get_type()));
                args.insert(0, var);
                runtime.push_stack(0, index as u16, args, file);
                FnResult::Ok(())
            }
            InnerMethod::Native(func) => runtime.call_native_method(func, callee, args),
            InnerMethod::Move(func) => runtime.call_copy_method(func, callee.clone(), args),
        }
    }
}

pub trait MethodClone {
    fn clone_box(&self) -> Box<dyn Method>;
}

pub trait Method: MethodClone + Debug {
    fn call(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult;
    fn call_or_goto(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult;
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
        left as *const () == right as *const ()
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

    pub fn new_native(
        value: T,
        method: fn(&T, Vec<Variable>, &mut Runtime) -> FnResult,
    ) -> Box<StdMethod<T>> {
        Box::new(StdMethod {
            value,
            method: InnerMethod::Native(method),
        })
    }

    pub fn new_move(
        value: T,
        method: fn(T, Vec<Variable>, &mut Runtime) -> FnResult,
    ) -> Box<StdMethod<T>> {
        Box::new(StdMethod {
            value,
            method: InnerMethod::Move(method),
        })
    }
}

impl<T: 'static + Debug> Method for StdMethod<T>
where
    T: Clone + Into<Variable>,
{
    fn call(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.method.call(&self.value, args.0, args.1)
    }

    fn call_or_goto(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.method.call_or_goto(&self.value, args.0, args.1)
    }
}

impl<T> Debug for InnerMethod<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerMethod::Standard(i, j) => f.debug_tuple("Standard").field(i).field(j).finish(),
            InnerMethod::Native(fn_) => f
                .debug_tuple("Native")
                .field(&format!("fn@{:#X}", *fn_ as usize))
                .finish(),
            InnerMethod::Move(fn_) => f
                .debug_tuple("Move")
                .field(&format!("fn@{:#X}", *fn_ as usize))
                .finish(),
        }
    }
}
