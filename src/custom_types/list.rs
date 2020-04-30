use crate::custom_types::types::CustomType;
use crate::custom_var::CustomVar;
use crate::function::Function;
use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use num::{BigInt, ToPrimitive};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct List {
    value: RefCell<Vec<Variable>>,
}

impl List {
    pub fn from_values(values: Vec<Variable>) -> Rc<List> {
        Rc::new(List {
            value: RefCell::new(values),
        })
    }

    fn get_operator(self: Rc<Self>, name: Operator) -> Variable {
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

    fn get_attribute(self: Rc<Self>, name: StringVar) -> Variable {
        let value = match name.as_str() {
            "length" => return Variable::Bigint(self.value.borrow().len().into()),
            "get" => Self::list_get,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), value))
    }

    fn list_bool(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push(Variable::Bool(!self.value.borrow().is_empty()));
        FnResult::Ok(())
    }

    fn list_str(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
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
        runtime.push(value.into());
        FnResult::Ok(())
    }

    fn list_index(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let index = BigInt::from(args[0].clone());
        if index > self.value.borrow().len().into() {
            runtime.throw(Variable::String("index out of range".into()))
        } else {
            runtime.push(self.value.borrow()[index.to_usize().unwrap()].clone());
            Result::Ok(())
        }
    }

    fn list_get(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let index = BigInt::from(args[0].clone());
        runtime.push(match index.to_usize() {
            Option::None => args[1].clone(),
            Option::Some(i) => match self.value.borrow().get(i) {
                Option::None => args[1].clone(),
                Option::Some(val) => val.clone(),
            },
        });
        FnResult::Ok(())
    }

    pub fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty()); // TODO: List of a value
        runtime.push(List::from_values(vec![]).into());
        FnResult::Ok(())
    }

    pub fn list_type() -> Type {
        lazy_static! {
            static ref TYPE: CustomType<List> = CustomType::new(
                "list".into(),
                Vec::new(),
                Function::Native(List::create),
                HashMap::new()
            );
        }
        Type::Custom(&*TYPE)
    }
}

impl CustomVar for List {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(o) => self.get_operator(o),
            Name::Attribute(s) => self.get_attribute(s),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        List::list_type()
    }
}
