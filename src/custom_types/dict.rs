use crate::custom_types::exceptions::key_error;
use crate::custom_types::inner_dict::{DictIter, DictLike, Entry, InnerDict};
use crate::custom_var::{downcast_var, CustomVar};
use crate::looping;
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::tuple::LangTuple;
use crate::variable::{FnResult, Variable};
use crate::{first, first_n};
use std::cell::{Ref, RefCell};
use std::fmt::Debug;
use std::mem::replace;
use std::rc::Rc;

#[derive(Debug)]
pub struct Dict {
    value: RefCell<InnerDict>,
}

impl Dict {
    pub fn new() -> Rc<Dict> {
        Rc::new(Dict {
            value: RefCell::new(InnerDict::new()),
        })
    }

    pub fn from_args(
        keys: Vec<Variable>,
        values: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> Result<Rc<Dict>, ()> {
        Result::Ok(Rc::new(Dict {
            value: RefCell::new(InnerDict::from_args(keys, values, runtime)?),
        }))
    }

    fn from_inner(value: InnerDict) -> Rc<Dict> {
        Rc::new(Dict {
            value: RefCell::new(value),
        })
    }

    fn op_fn(o: Operator) -> NativeMethod<Rc<Dict>> {
        match o {
            Operator::GetAttr => Dict::index,
            Operator::Repr => Dict::repr,
            Operator::Str => Dict::repr,
            Operator::Bool => Dict::bool,
            Operator::SetAttr => Dict::set,
            Operator::In => Dict::contains,
            Operator::Equals => Dict::eq,
            Operator::Iter => Dict::iter,
            Operator::DelAttr => Dict::del,
            _ => unimplemented!("dict.{}", o.name()),
        }
    }

    fn attr_fn(s: &str) -> NativeMethod<Rc<Dict>> {
        match s {
            "clear" => Dict::clear,
            "get" => Dict::get,
            "getPair" => Dict::get_pair,
            "replace" => Dict::replace,
            "remove" => Dict::remove,
            "setDefault" => Dict::set_default,
            "retain" => Dict::retain,
            _ => unimplemented!("dict.{}", s),
        }
    }

    fn index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        match self.value.borrow().get(first(args), runtime)? {
            Option::Some(result) => runtime.return_1(result),
            Option::None => runtime.throw_quick(key_error(), "Value not found"),
        }
    }

    fn repr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let repr = self.value.borrow().true_repr(runtime)?;
        runtime.return_1(repr.into())
    }

    fn bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.is_empty()).into())
    }

    fn set(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let [key, val] = first_n(args);
        self.value.borrow_mut().set(key, val, runtime)?;
        runtime.return_0()
    }

    fn contains(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let is_in = self.value.borrow().get(first(args), runtime)?.is_some();
        runtime.return_1(is_in.into())
    }

    fn del(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        self.value.borrow_mut().del(first(args), runtime)?;
        runtime.return_0()
    }

    fn clear(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().clear();
        runtime.return_0()
    }

    fn get(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        if args.len() == 1 {
            let val = self.value.borrow().get(first(args), runtime)?.into();
            runtime.return_1(val)
        } else {
            debug_assert_eq!(args.len(), 2);
            let [key, default] = first_n(args);
            let val = self.value.borrow().get(key, runtime)?.unwrap_or(default);
            runtime.return_1(val)
        }
    }

    fn get_pair(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let key = first(args);
        let val = self.value.borrow().get_pair(key, runtime)?;
        let mapped = val.map(|(x, y)| LangTuple::from_vec(vec![x, y]).into());
        runtime.return_1(mapped.into())
    }

    fn replace(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let [key, val] = first_n(args);
        let mut value = self.value.borrow_mut();
        if value.is_empty() {
            runtime.return_1(Option::None.into())
        } else if let Result::Ok(e) = value.entry_mut(key, runtime)?.into_value() {
            runtime.return_1(Option::Some(replace(e.value_mut(), val)).into())
        } else {
            runtime.return_1(Option::None.into())
        }
    }

    fn remove(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let returned = self.value.borrow_mut().del(first(args), runtime)?.into();
        runtime.return_1(returned)
    }

    fn set_default(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let mut value = self.value.borrow_mut();
        let [arg, default] = first_n(args);
        value.resize(1);
        let result = match value.entry_mut(arg.clone(), runtime)?.into_value() {
            Result::Ok(e) => e.clone_value(),
            Result::Err(mut e) => {
                let hash = arg.clone().hash(runtime)?;
                e.put(arg, default.clone(), hash);
                default
            }
        };
        runtime.return_1(result)
    }

    fn retain(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let predicate = first(args);
        let mut removed = 0;
        let mut value = self.value.borrow_mut();
        for entry in value.entries_raw_mut() {
            if let Entry::Some(e) = entry {
                predicate
                    .clone()
                    .call_or_goto((vec![e.clone_key(), e.clone_value()], runtime))?;
                let result = runtime.pop_return().into_bool(runtime)?;
                if !result {
                    *entry = Entry::Removed;
                    removed += 1;
                }
            }
        }
        *value.size_mut() -= removed;
        runtime.return_0()
    }

    fn eq(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            match downcast_var::<Dict>(arg) {
                Result::Err(_) => return runtime.return_1(false.into()),
                Result::Ok(other) => {
                    let self_val = self.value.borrow();
                    if !self_val.equals(&*other.value.borrow(), runtime)? {
                        return runtime.return_1(false.into());
                    }
                }
            };
        }
        runtime.return_1(true.into())
    }

    fn iter(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(DictIter::new(self)).into())
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let dict = match args.len() {
            0 => Dict::new(),
            1 => {
                let value = first(args);
                match downcast_var::<Dict>(value) {
                    Result::Ok(x) => Dict::from_inner(x.value.borrow().clone()),
                    Result::Err(x) => {
                        let iter = x.iter(runtime)?;
                        let mut inner = InnerDict::new();
                        while let Option::Some([key, val]) = iter.next(runtime)?.take_n() {
                            inner.set(key, val, runtime)?;
                        }
                        Dict::from_inner(inner)
                    }
                }
            }
            x => panic!("Expected 0 or 1 arguments for dict.operator new, got {}", x),
        };
        runtime.return_1(dict.into())
    }

    pub fn dict_type() -> Type {
        custom_class!(Dict, create, "dict")
    }

    fn is_empty(&self) -> bool {
        self.value.borrow().is_empty()
    }

    fn len(&self) -> usize {
        self.value.borrow().size()
    }
}

impl CustomVar for Dict {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Dict::dict_type()
    }

    fn get_operator(self: Rc<Self>, o: Operator) -> Variable {
        let func = Dict::op_fn(o);
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, s: &str) -> Variable {
        let func = match s {
            "length" => return self.len().into(),
            _ => Self::attr_fn(s),
        };
        StdMethod::new_native(self, func).into()
    }

    fn call_op(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(Dict::op_fn(operator), self, args)
    }

    fn call_op_or_goto(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(Dict::op_fn(operator), self, args)
    }

    fn str(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.value.borrow().true_repr(runtime)
    }

    fn repr(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.value.borrow().true_repr(runtime)
    }

    fn bool(self: Rc<Self>, _runtime: &mut Runtime) -> Result<bool, ()> {
        Result::Ok(!self.value.borrow().is_empty())
    }

    fn iter(self: Rc<Self>, _runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        Result::Ok(Rc::new(DictIter::new(self)).into())
    }
}

impl DictLike for Dict {
    fn borrow(&self) -> Ref<'_, InnerDict> {
        self.value.borrow()
    }
}
