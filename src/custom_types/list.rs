use crate::custom_types::types::CustomType;
use crate::custom_var::{downcast_var, CustomVar};
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
            Operator::Equals => List::eq,
            Operator::Iter => List::iter,
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

    fn eq(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            if !match downcast_var::<List>(arg) {
                Option::None => false,
                Option::Some(other) => {
                    let self_val = self.value.borrow();
                    let other_val = other.value.borrow();
                    self_val.len() == other_val.len()
                        && Self::vec_eq(&*self_val, &*other_val, runtime)?
                }
            } {
                runtime.push(false.into());
                return FnResult::Ok(());
            }
        }
        runtime.push(true.into());
        FnResult::Ok(())
    }

    fn vec_eq(
        first: &Vec<Variable>,
        second: &Vec<Variable>,
        runtime: &mut Runtime,
    ) -> Result<bool, ()> {
        let mut is_eq = true;
        for (a, b) in first.iter().zip(second.iter()) {
            if !a.equals(b.clone(), runtime)? {
                is_eq = false;
                break;
            }
        }
        Result::Ok(is_eq)
    }

    fn iter(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push(Rc::new(ListIter::new(self.clone())).into());
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

#[derive(Debug)]
struct ListIter {
    current: RefCell<usize>,
    value: Rc<List>,
}

impl ListIter {
    pub fn new(value: Rc<List>) -> ListIter {
        ListIter {
            current: RefCell::new(0),
            value,
        }
    }

    fn get_attribute(self: &Rc<Self>, val: StringVar) -> Variable {
        let func = match val.as_str() {
            "next" => Self::next_fn,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn next_fn(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.next() {
            Option::Some(value) => {
                runtime.push(value.into());
                FnResult::Ok(())
            }
            Option::None => runtime.throw_quick(Type::String, "".into()),
        }
    }

    fn next(&self) -> Option<Variable> {
        if *self.current.borrow() != self.value.value.borrow().len() {
            let result = self.value.value.borrow()[*self.current.borrow()].clone();
            *self.current.borrow_mut() += 1;
            Option::Some(result)
        } else {
            Option::None
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn range_iter_type() -> Type {
        lazy_static! {
            static ref TYPE: CustomType<ListIter> = CustomType::new(
                "list".into(),
                Vec::new(),
                Function::Native(ListIter::create),
                HashMap::new()
            );
        }
        Type::Custom(&*TYPE)
    }
}

impl CustomVar for ListIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(_) => unimplemented!(),
            Name::Attribute(s) => self.get_attribute(s),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::range_iter_type()
    }
}
