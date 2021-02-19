use crate::custom_var::CustomVar;
use crate::first;
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use crate::variable::{FnResult, OptionVar, Variable};
use std::iter::FromIterator;
use std::iter::Iterator as stdIterator;
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

pub trait IterAttrs: NativeIterator + Sized {
    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult;
    fn get_type() -> Type;

    fn get_name(self: Rc<Self>, val: &str) -> Variable {
        let func = match val {
            "next" => Self::next_fn,
            _ => unimplemented!("{}", val),
        };
        StdMethod::new_native(self, func).into()
    }

    fn get_op(self: Rc<Self>, val: Operator) -> Variable {
        let func = match val {
            Operator::Iter => Self::ret_self,
            _ => unimplemented!("{}", val.name()),
        };
        StdMethod::new_native(self, func).into()
    }

    fn ret_self(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.into())
    }
}

pub trait TypicalIterator: IterAttrs + NativeIterator + Sized {
    fn inner_next(&self) -> Option<Variable>;
    fn get_type() -> Type;
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

pub fn collect<T, U>(value: Variable, runtime: &mut Runtime) -> Result<T, ()>
where
    T: FromIterator<U>,
    U: From<Variable>,
{
    IterAdaptor {
        value: Result::Ok(value.iter(runtime)?),
        runtime,
    }
    .map(|x| x.map(U::from))
    .collect()
}

impl<T> From<Rc<T>> for Iterator
where
    T: NativeIterator,
{
    fn from(x: Rc<T>) -> Self {
        Iterator::Native(x)
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
            IterOk::Vec(v) => v.map(first),
        }
    }
}

impl<T> NativeIterator for T
where
    T: TypicalIterator,
{
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.inner_next().into())
    }
}

impl<T> CustomVar for T
where
    T: IterAttrs,
{
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        <Self as IterAttrs>::get_type()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        self.get_op(op)
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        self.get_name(name)
    }
}

impl<T> IterAttrs for T
where
    T: TypicalIterator,
{
    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.inner_next().into())
    }

    fn get_type() -> Type {
        <Self as TypicalIterator>::get_type()
    }
}

struct IterAdaptor<'a> {
    value: Result<Iterator, bool>,
    runtime: &'a mut Runtime,
}

impl std::iter::Iterator for IterAdaptor<'_> {
    type Item = Result<Variable, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.value {
            Result::Ok(value) => match value.next(self.runtime).map(IterOk::take_first) {
                Result::Ok(Option::Some(val)) => Option::Some(Result::Ok(val)),
                Result::Ok(Option::None) => {
                    self.value = Result::Err(false);
                    Option::None
                }
                Result::Err(_) => {
                    self.value = Result::Err(true);
                    Option::Some(Result::Err(()))
                }
            },
            // Safeguard against multiple calls to completed iterator
            Result::Err(true) => Option::Some(Result::Err(())),
            Result::Err(false) => Option::None,
        }
    }
}
