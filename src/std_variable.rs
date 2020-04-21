use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::vec::Vec;

use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::{StdType, Type};
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};

pub type StdVarMethod = InnerMethod<StdVariable>;

#[derive(Debug, Clone, Eq)]
pub struct StdVariable {
    value: Rc<RefCell<InnerVar>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InnerVar {
    pub cls: &'static StdType,
    pub values: HashMap<Name, Variable>,
}

impl StdVariable {
    pub fn new(cls: &'static StdType, values: HashMap<Name, Variable>) -> StdVariable {
        StdVariable {
            value: Rc::new(RefCell::new(InnerVar::new(cls, values))),
        }
    }

    pub fn str(&mut self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.call_operator(Operator::Str, vec![], runtime)?;
        runtime.pop().str(runtime)
    }

    pub fn bool(&mut self, runtime: &mut Runtime) -> Result<bool, ()> {
        self.call_operator(Operator::Bool, vec![], runtime)?;
        runtime.pop().to_bool(runtime)
    }

    pub fn call_operator(
        &self,
        op: Operator,
        mut args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        let inner_method = self.value.borrow().cls.get_method(Name::Operator(op));
        return match inner_method {
            StdVarMethod::Standard(fn_no) => {
                let var: Variable = Variable::Standard(self.clone());
                args.reserve(2);
                args.insert(0, Variable::Type(var.get_type()));
                args.insert(0, var);
                runtime.push_stack(0, fn_no as u16, args, 0)?;
                FnResult::Ok(())
            }
            StdVarMethod::Native(func) => {
                runtime.push_native();
                let result = func(self, args, runtime);
                runtime.pop_native();
                result
            }
        };
    }

    pub fn call(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        self.call_operator(Operator::Call, args.0, args.1)
    }

    pub fn index(&self, index: Name) -> Variable {
        let self_value = self.value.borrow();
        let val = self_value.values.get(&index);
        match val {
            Option::Some(true_val) => true_val.clone(),
            Option::None => {
                let inner_method = self.value.borrow().cls.get_method(index);
                Variable::Method(Box::new(StdMethod::new(self.clone(), inner_method)))
            }
        }
    }

    pub fn set(&self, index: StringVar, value: Variable) {
        self.value
            .borrow_mut()
            .values
            .insert(Name::Attribute(index), value);
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
    fn new(cls: &'static StdType, values: HashMap<Name, Variable>) -> InnerVar {
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
