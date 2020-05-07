use crate::int_var::IntVar;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use downcast_rs::Downcast;
use std::any::Any;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

pub trait CustomVar: Debug + Any + Downcast {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable;
    fn set(self: Rc<Self>, name: Name, object: Variable);
    fn get_type(self: Rc<Self>) -> Type;

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
        runtime.pop().str(runtime)
    }

    fn int(self: Rc<Self>, runtime: &mut Runtime) -> Result<IntVar, ()> {
        self.call_op(Operator::Int, vec![], runtime)?;
        runtime.pop().int(runtime)
    }

    fn bool(self: Rc<Self>, runtime: &mut Runtime) -> Result<bool, ()> {
        self.call_op(Operator::Bool, vec![], runtime)?;
        runtime.pop().to_bool(runtime)
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
}

impl Deref for CustomVarWrapper {
    type Target = Rc<dyn CustomVar>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Hash for CustomVarWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ref(&self.value), state)
    }
}

impl PartialEq for CustomVarWrapper {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

impl Eq for CustomVarWrapper {}

impl From<Rc<dyn CustomVar>> for Variable {
    fn from(x: Rc<dyn CustomVar>) -> Self {
        Variable::Custom(CustomVarWrapper::new(x))
    }
}

impl<T> From<Rc<T>> for Variable
where
    T: CustomVar + 'static,
{
    fn from(val: Rc<T>) -> Self {
        Variable::Custom(CustomVarWrapper::new(val))
    }
}

pub fn downcast_var<T>(var: Variable) -> Option<Rc<T>>
where
    T: 'static + CustomVar,
{
    if let Variable::Custom(wrapper) = var {
        (*wrapper).clone().downcast_rc::<T>().ok()
    } else {
        Option::None
    }
}
