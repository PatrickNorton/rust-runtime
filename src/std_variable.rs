use crate::int_var::IntVar;
use crate::looping;
use crate::method::{InnerMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::{StdType, Type};
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

pub type StdVarMethod = InnerMethod<StdVariable>;

#[derive(Debug, Clone, Eq)]
pub struct StdVariable {
    value: Rc<RefCell<InnerVar>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InnerVar {
    pub cls: &'static StdType,
    pub values: HashMap<Arc<str>, Variable>,
}

impl StdVariable {
    pub fn new(cls: &'static StdType, values: HashMap<Arc<str>, Variable>) -> StdVariable {
        StdVariable {
            value: Rc::new(RefCell::new(InnerVar::new(cls, values))),
        }
    }

    pub fn str(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_operator(Operator::Str, vec![], runtime)?;
        runtime.pop_return().str(runtime)
    }

    pub fn repr(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_operator(Operator::Repr, Vec::new(), runtime)?;
        Result::Ok(runtime.pop_return().into())
    }

    pub fn bool(self, runtime: &mut Runtime) -> Result<bool, ()> {
        self.call_operator(Operator::Bool, vec![], runtime)?;
        runtime.pop_return().into_bool(runtime)
    }

    pub fn int(self, runtime: &mut Runtime) -> Result<IntVar, ()> {
        self.call_operator(Operator::Bool, vec![], runtime)?;
        runtime.pop_return().int(runtime)
    }

    pub fn iter(self, runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        self.call_operator(Operator::Bool, vec![], runtime)?;
        Result::Ok(runtime.pop_return().into())
    }

    pub fn call_operator(
        self,
        op: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        let inner_method = self
            .value
            .borrow()
            .cls
            .get_method(Name::Operator(op), runtime);
        inner_method.call(self.clone(), args, runtime)
    }

    pub fn call(self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.call_operator(Operator::Call, args.0, args.1)
    }

    pub fn call_op_or_goto(
        &self,
        op: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        let inner_method = self
            .value
            .borrow()
            .cls
            .get_method(Name::Operator(op), runtime);
        inner_method.call_or_goto(self, args, runtime)
    }

    pub fn call_or_goto(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.call_op_or_goto(Operator::Call, args.0, args.1)
    }

    pub fn index(&self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        let self_value = self.value.borrow();
        let val = match index {
            Name::Attribute(a) => self_value.values.get(a),
            Name::Operator(_) => Option::None,
        };
        match val {
            Option::Some(true_val) => Result::Ok(true_val.clone()),
            Option::None => self.index_harder(index, runtime),
        }
    }

    fn index_harder(&self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        match self.value.borrow().cls.get_property(index) {
            Option::Some(val) => {
                val.call_getter(runtime, self.clone().into())?;
                Result::Ok(runtime.pop_return())
            }
            Option::None => {
                let inner_method = self.value.borrow().cls.get_method(index, runtime);
                Result::Ok(Box::new(StdMethod::new(self.clone(), inner_method)).into())
            }
        }
    }

    pub fn set(&self, index: &str, value: Variable, runtime: &mut Runtime) -> FnResult {
        let mut self_val = self.value.borrow_mut();
        match self_val.values.get_mut(index) {
            Option::Some(val) => *val = value,
            Option::None => {
                drop(self_val); // Will cause double-mutable borrow otherwise
                let self_val = self.value.borrow();
                match self_val.cls.get_property(Name::Attribute(index)) {
                    Option::Some(val) => {
                        drop(self_val); // Ditto
                        val.call_setter(runtime, self.clone().into(), value)?
                    }
                    Option::None => unimplemented!(
                        "{}.{}\n{}",
                        self.get_type().str(),
                        index,
                        runtime.stack_frames()
                    ),
                }
            }
        }
        runtime.return_0()
    }

    pub fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }

    pub fn get_type(&self) -> Type {
        Type::Standard(self.value.borrow().cls)
    }

    pub fn var_ptr(&self) -> usize {
        self.value.as_ptr() as usize
    }
}

impl InnerVar {
    fn new(cls: &'static StdType, values: HashMap<Arc<str>, Variable>) -> InnerVar {
        InnerVar { cls, values }
    }
}

impl Hash for StdVariable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i128(self.value.as_ptr() as i128)
    }
}

impl PartialEq for StdVariable {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}
