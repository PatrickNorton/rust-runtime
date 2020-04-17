use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::{StdType, Type};
use crate::variable::{Name, Variable};

pub type StdVarMethod = InnerMethod<StdVariable>;

#[derive(Clone, Eq)]
pub struct StdVariable {
    value: Rc<RefCell<InnerVar>>,
}

#[derive(Clone, PartialEq, Eq)]
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

    pub fn str(&mut self, runtime: &mut Runtime) -> String {
        self.call_operator(Operator::Str, runtime);
        return runtime.pop().str(runtime);
    }

    pub fn bool(&mut self, runtime: &mut Runtime) -> bool {
        self.call_operator(Operator::Bool, runtime);
        runtime.pop().to_bool(runtime)
    }

    pub fn call_operator(&mut self, _op: Operator, _runtime: &mut Runtime) {
        unimplemented!()
    }

    pub fn call(&self, args: (Vec<Variable>, &mut Runtime)) {
        self.value.borrow_mut().values[&Name::Operator(Operator::Call)].call(args)
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

    pub fn set(&self, index: String, value: Variable) {
        self.value
            .borrow_mut()
            .values
            .insert(Name::Attribute(index), value);
    }

    pub fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }

    pub fn get_type(&self) -> Type {
        Type::Standard(self.value.borrow_mut().cls)
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
