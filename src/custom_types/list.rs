use crate::custom_var::CustomVar;
use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::variable::{FnResult, Name, Variable};
use num::{BigInt, ToPrimitive};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct List {
    value: Rc<RefCell<Vec<Variable>>>,
}

impl List {
    pub fn from_values(values: Vec<Variable>) -> List {
        List {
            value: Rc::new(RefCell::new(values)),
        }
    }

    fn get_operator(&self, name: Operator) -> Variable {
        let value = match name {
            Operator::Bool => List::list_bool,
            Operator::Str => List::list_str,
            Operator::GetAttr => List::list_index,
            _ => unimplemented!(),
        };
        Variable::Method(Box::new(StdMethod::new(
            self.clone(),
            InnerMethod::Native(value),
        )))
    }

    fn list_bool(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push(Variable::Bool(!self.value.borrow().is_empty()));
        FnResult::Ok(())
    }

    fn list_str(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let mut value: String = String::new();
        value += "[";
        for arg in self.value.borrow().iter().enumerate() {
            value += arg.1.str(runtime)?.as_str();
            if arg.0 != self.value.borrow().len() - 1 {
                value += ", ";
            }
        }
        value += "]";
        FnResult::Ok(())
    }

    fn list_index(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let index = BigInt::from(args[0].clone());
        if index > self.value.borrow().len().into() {
            runtime.throw(Variable::String("index out of range".into()))
        } else {
            runtime.push(self.value.borrow()[index.to_usize().unwrap()].clone());
            Result::Ok(())
        }
    }
}

impl CustomVar for List {
    fn get_attr(&self, name: Name) -> Variable {
        return match name {
            Name::Operator(o) => self.get_operator(o),
            Name::Attribute(_) => unimplemented!(),
        };
    }

    fn set(&mut self, name: Name, object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        unimplemented!()
    }
}

impl Into<Variable> for List {
    fn into(self) -> Variable {
        Box::new(self).into()
    }
}
