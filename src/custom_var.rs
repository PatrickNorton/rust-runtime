use crate::int_var::IntVar;
use crate::looping;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, InnerVar, Variable};
use downcast_rs::Downcast;
use std::any::Any;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

pub trait CustomVar: Debug + Any + Downcast {
    fn set(self: Rc<Self>, name: Name, object: Variable);
    fn get_type(&self) -> Type;

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable;
    fn get_attribute(self: Rc<Self>, name: &str) -> Variable;

    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Attribute(a) => self.get_attribute(a),
            Name::Operator(o) => self.get_operator(o),
        }
    }

    fn call(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.call_op(Operator::Call, args, runtime)
    }

    fn call_or_goto(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.call_op_or_goto(Operator::Call, args, runtime)
    }

    fn call_op(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        self.get_attr(Name::Operator(operator))
            .call((args, runtime))
    }

    fn call_op_or_goto(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        self.get_attr(Name::Operator(operator))
            .call_or_goto((args, runtime))
    }

    fn str(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_op(Operator::Str, vec![], runtime)?;
        runtime.pop_return().str(runtime)
    }

    fn repr(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_op(Operator::Repr, Vec::new(), runtime)?;
        Result::Ok(runtime.pop_return().into())
    }

    fn int(self: Rc<Self>, runtime: &mut Runtime) -> Result<IntVar, ()> {
        self.call_op(Operator::Int, vec![], runtime)?;
        runtime.pop_return().int(runtime)
    }

    fn bool(self: Rc<Self>, runtime: &mut Runtime) -> Result<bool, ()> {
        self.call_op(Operator::Bool, vec![], runtime)?;
        runtime.pop_return().into_bool(runtime)
    }

    fn iter(self: Rc<Self>, runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        self.call_op(Operator::Iter, vec![], runtime)?;
        Result::Ok(runtime.pop_return().into())
    }

    fn into_iter(self: Rc<Self>) -> looping::Iterator {
        panic!(
            "Cannot into_iter a non-iterable (value has type {})",
            self.get_type().str()
        )
    }
}

impl_downcast!(CustomVar);

#[derive(Debug, Clone)]
pub struct CustomVarWrapper {
    value: Rc<dyn CustomVar>,
}

impl CustomVarWrapper {
    pub fn new(value: Rc<dyn CustomVar>) -> CustomVarWrapper {
        CustomVarWrapper { value }
    }

    pub fn into_inner(self) -> Rc<dyn CustomVar> {
        self.value
    }
}

impl Deref for CustomVarWrapper {
    type Target = dyn CustomVar;

    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}

impl Hash for CustomVarWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ref(&self.value), state)
    }
}

impl PartialEq for CustomVarWrapper {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(
            self.value.as_ref() as *const dyn CustomVar as *const (),
            other.value.as_ref() as *const dyn CustomVar as *const (),
        )
    }
}

impl Eq for CustomVarWrapper {}

impl From<Rc<dyn CustomVar>> for Variable {
    fn from(x: Rc<dyn CustomVar>) -> Self {
        Variable::Normal(InnerVar::Custom(CustomVarWrapper::new(x)))
    }
}

impl<T> From<Rc<T>> for Variable
where
    T: CustomVar,
{
    fn from(val: Rc<T>) -> Self {
        Variable::Normal(InnerVar::Custom(CustomVarWrapper::new(val)))
    }
}

pub fn downcast_var<T>(var: Variable) -> Result<Rc<T>, Variable>
where
    T: 'static + CustomVar,
{
    if let Variable::Normal(InnerVar::Custom(wrapper)) = var {
        wrapper.into_inner().downcast_rc::<T>().map_err(Into::into)
    } else {
        Result::Err(var)
    }
}
