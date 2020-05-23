use crate::custom_types::exceptions::{index_error, stop_iteration};
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_var::IntVar;
use crate::looping;
use crate::looping::{IterResult, NativeIterator};
use crate::method::{InnerMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use num::{Signed, ToPrimitive};
use std::cell::{Cell, RefCell};
use std::mem::replace;
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
            Operator::In => List::contains,
            _ => unimplemented!(),
        };
        Variable::Method(Box::new(StdMethod::new(self, InnerMethod::Native(value))))
    }

    fn get_attribute(self: Rc<Self>, name: StringVar) -> Variable {
        let value = match name.as_str() {
            "length" => return Variable::Bigint(self.value.borrow().len().into()),
            "contains_all" => Self::contains_all,
            "get" => Self::list_get,
            "reverse" => Self::reverse,
            "count" => Self::count,
            "clear" => Self::clear,
            "add" => Self::add,
            _ => unimplemented!("List::{}", name),
        };
        Variable::Method(StdMethod::new_native(self, value))
    }

    fn list_bool(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Variable::Bool(!self.value.borrow().is_empty()))
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
        runtime.return_1(value.into())
    }

    fn list_index(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let signed_index = IntVar::from(args[0].clone());
        let index = if signed_index.is_negative() {
            signed_index + self.value.borrow().len().into()
        } else {
            signed_index
        };
        if index >= self.value.borrow().len().into() || index.is_negative() {
            runtime.throw_quick(
                index_error(),
                format!(
                    "index {} out of range for list of length {}",
                    index,
                    self.value.borrow().len()
                )
                .into(),
            )
        } else {
            runtime.push(self.value.borrow()[index.to_usize().unwrap()].clone());
            Result::Ok(())
        }
    }

    fn list_get(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let index = IntVar::from(args[0].clone());
        runtime.push(match index.to_usize() {
            Option::None => args[1].clone(),
            Option::Some(i) => match self.value.borrow().get(i) {
                Option::None => args[1].clone(),
                Option::Some(val) => val.clone(),
            },
        });
        FnResult::Ok(())
    }

    fn contains(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.return_1(self.value.borrow().contains(&args[0]).into())
    }

    fn contains_all(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let checked_var = replace(&mut args[0], Variable::Null());
        let this_iter = checked_var.iter(runtime)?;
        while let Option::Some(val) = this_iter.clone().next(runtime)? {
            if !self.value.borrow().contains(&val) {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn reverse(self: &Rc<Self>, args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().reverse();
        FnResult::Ok(())
    }

    fn count(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let mut count: usize = 0;
        for x in &*self.value.borrow() {
            if x.equals(self.clone().into(), runtime)? {
                count += 1;
            }
        }
        runtime.return_1(IntVar::from(count).into())
    }

    fn clear(self: &Rc<Self>, args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().clear();
        FnResult::Ok(())
    }

    fn add(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        self.value
            .borrow_mut()
            .push(replace(&mut args[0], Variable::Null()));
        runtime.return_0()
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
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn vec_eq(first: &[Variable], second: &[Variable], runtime: &mut Runtime) -> Result<bool, ()> {
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
        runtime.return_1(Rc::new(ListIter::new(self.clone())).into())
    }

    pub fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty()); // TODO: List of a value
        runtime.return_1(List::from_values(vec![]).into())
    }

    pub fn list_type() -> Type {
        custom_class!(List, create, "list")
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
    current: Cell<usize>,
    value: Rc<List>,
}

impl ListIter {
    pub fn new(value: Rc<List>) -> ListIter {
        ListIter {
            current: Cell::new(0),
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
        match self.inner_next() {
            Option::Some(value) => runtime.return_1(value),
            Option::None => runtime.throw_quick(stop_iteration(), "".into()),
        }
    }

    fn inner_next(&self) -> Option<Variable> {
        if self.current.get() != self.value.value.borrow().len() {
            let result = self.value.value.borrow()[self.current.get()].clone();
            self.current.set(self.current.get() + 1);
            Option::Some(result)
        } else {
            Option::None
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn range_iter_type() -> Type {
        custom_class!(ListIter, create, "ListIter")
    }
}

impl CustomVar for ListIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        name.do_each(|_| unimplemented!(), |s| self.get_attribute(s))
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::range_iter_type()
    }

    fn into_iter(self: Rc<Self>) -> looping::Iterator {
        looping::Iterator::Native(self)
    }
}

impl NativeIterator for ListIter {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.inner_next())
    }
}
