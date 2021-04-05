use crate::custom_types::dict::{DictIter, DictLike, InnerDict};
use crate::custom_var::{downcast_var, CustomVar};
use crate::first;
use crate::int_var::IntVar;
use crate::looping;
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::cell::{Ref, RefCell};
use std::rc::Rc;

#[derive(Debug)]
pub struct Set {
    generic: Type,
    value: RefCell<InnerDict>,
}

impl Set {
    pub fn new(generic: Type, args: Vec<Variable>, runtime: &mut Runtime) -> Result<Rc<Set>, ()> {
        let len = args.len();
        Result::Ok(Rc::new(Set {
            generic,
            value: RefCell::new(InnerDict::from_args(
                args,
                vec![Variable::null(); len],
                runtime,
            )?),
        }))
    }

    fn from_inner(generic: Type, value: InnerDict) -> Rc<Set> {
        Rc::new(Set {
            generic,
            value: RefCell::new(value),
        })
    }

    fn op_fn(o: Operator) -> NativeMethod<Rc<Set>> {
        match o {
            Operator::Bool => Self::bool,
            Operator::Str => Self::repr,
            Operator::Repr => Self::repr,
            Operator::Iter => Self::iter,
            Operator::In => Self::contains,
            Operator::Equals => Self::eq,
            Operator::BitwiseAnd => Self::intersection,
            Operator::BitwiseOr => Self::union,
            Operator::BitwiseXor => Self::xor,
            Operator::DelAttr => Self::del_attr,
            _ => unimplemented!(),
        }
    }

    fn attr_fn(s: &str) -> NativeMethod<Rc<Set>> {
        match s {
            "add" => Self::add,
            "addAll" => Self::add_all,
            "remove" => Self::remove,
            "clear" => Self::clear,
            "isSubset" => Self::subset,
            "isSuperset" => Self::superset,
            "isDisjoint" => Self::disjoint,
            "containsAll" => Self::contains_all,
            _ => unimplemented!(),
        }
    }

    fn intersection(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let other = first(args);
        let other_iter = other.iter(runtime)?;
        let mut result_vec = Vec::new();
        while let Option::Some(val) = other_iter.next(runtime)?.take_first() {
            if self.value.borrow().get(val.clone(), runtime)?.is_some() {
                result_vec.push(val);
            }
        }
        let ret = Self::new(self.generic, result_vec, runtime)?;
        runtime.return_1(ret.into())
    }

    fn union(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let mut result = (*self.value.borrow()).clone();
        let other = first(args);
        let other_iter = other.iter(runtime)?;
        while let Option::Some(val) = other_iter.next(runtime)?.take_first() {
            result.set(val, Variable::null(), runtime)?;
        }
        runtime.return_1(Set::from_inner(self.generic, result).into())
    }

    fn xor(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let mut result = (*self.value.borrow()).clone();
        let other = first(args);
        let other_iter = other.iter(runtime)?;
        while let Option::Some(val) = other_iter.next(runtime)?.take_first() {
            if result.get(val.clone(), runtime)?.is_some() {
                result.del(val, runtime)?;
            } else {
                result.set(val, Variable::null(), runtime)?;
            }
        }
        runtime.return_1(Set::from_inner(self.generic, result).into())
    }

    fn bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.is_empty()).into())
    }

    fn repr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let repr = self.value.borrow().key_repr(runtime)?;
        runtime.return_1(repr.into())
    }

    fn contains(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = first(args);
        let is_contained = self.value.borrow().get(val, runtime)?.is_some();
        runtime.return_1(is_contained.into())
    }

    fn add(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = first(args);
        if val.get_type().is_subclass(&self.generic, runtime) {
            self.value
                .borrow_mut()
                .set(val, Variable::null(), runtime)?;
        } else {
            panic!(
                "Bad type for set.add: {} is not a superclass of {}",
                val.get_type().str(),
                &self.generic.str()
            )
        }
        runtime.return_0()
    }

    fn add_all(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = first(args);
        let val_iter = val.iter(runtime)?;
        while let Option::Some(arg) = val_iter.next(runtime)?.take_first() {
            if arg.get_type().is_subclass(&self.generic, runtime) {
                self.value
                    .borrow_mut()
                    .set(arg, Variable::null(), runtime)?;
            } else {
                panic!(
                    "Bad type for set.addAll: {} is not a superclass of {}",
                    arg.get_type().str(),
                    &self.generic.str()
                )
            }
        }
        runtime.return_0()
    }

    fn clear(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().clear();
        runtime.return_0()
    }

    fn del_attr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        self.value.borrow_mut().del(first(args), runtime)?;
        runtime.return_0()
    }

    fn remove(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = first(args);
        let was_removed = self.value.borrow_mut().del(val, runtime)?.is_some();
        runtime.return_1(was_removed.into())
    }

    fn eq(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            if !match downcast_var::<Set>(arg) {
                Result::Err(_) => false,
                Result::Ok(other) => {
                    let self_val = self.value.borrow();
                    self_val.equals(&*other.value.borrow(), runtime)?
                }
            } {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn subset(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let other = downcast_var::<Set>(first(args)).expect("Expected a set");
        let other_val = other.value.borrow();
        for (value, _) in &*self.value.borrow() {
            if other_val.get(value.clone(), runtime)?.is_none() {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn superset(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let other = downcast_var::<Set>(first(args)).expect("Expected a set");
        let self_val = self.value.borrow();
        for (value, _) in &*other.value.borrow() {
            if self_val.get(value.clone(), runtime)?.is_none() {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn disjoint(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let other = downcast_var::<Set>(first(args)).expect("Expected a set");
        let self_val = self.value.borrow();
        for (value, _) in &*other.value.borrow() {
            if self_val.get(value.clone(), runtime)?.is_some() {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn contains_all(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let iter = first(args).iter(runtime)?;
        while let Option::Some(val) = iter.next(runtime)?.take_first() {
            if self.value.borrow().get(val, runtime)?.is_none() {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn iter(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(DictIter::new(self)).into())
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let set = match args.len() {
            0 => Set::from_inner(Type::Object, InnerDict::new()),
            1 => match downcast_var::<Set>(first(args)) {
                Result::Ok(x) => Set::from_inner(x.generic, x.value.borrow().clone()),
                Result::Err(arg) => {
                    let mut inner = InnerDict::new();
                    let iter = arg.iter(runtime)?;
                    while let Option::Some(val) = iter.next(runtime)?.take_first() {
                        inner.set(val, Variable::null(), runtime)?;
                    }
                    // TODO: Generic value
                    Set::from_inner(Type::Object, inner)
                }
            },
            x => unimplemented!(
                "set.operator new expected 0 or 1 args, got {}\n{}",
                x,
                runtime.frame_strings()
            ),
        };
        runtime.return_1(set.into())
    }

    pub fn set_type() -> Type {
        custom_class!(Set, create, "set")
    }

    pub fn is_empty(&self) -> bool {
        self.value.borrow().is_empty()
    }

    pub fn len(&self) -> usize {
        self.value.borrow().size()
    }
}

impl CustomVar for Set {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::set_type()
    }

    fn get_operator(self: Rc<Self>, o: Operator) -> Variable {
        let func = Self::op_fn(o);
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        match name {
            "length" => self.len().into(),
            _ => StdMethod::new_native(self, Self::attr_fn(name)).into(),
        }
    }

    fn call_op(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(Set::op_fn(operator), self, args)
    }

    fn call_op_or_goto(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(Set::op_fn(operator), self, args)
    }

    fn str(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.value.borrow().key_repr(runtime)
    }

    fn repr(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.value.borrow().key_repr(runtime)
    }

    fn bool(self: Rc<Self>, _runtime: &mut Runtime) -> Result<bool, ()> {
        Result::Ok(!self.value.borrow().is_empty())
    }

    fn iter(self: Rc<Self>, _runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        Result::Ok(Rc::new(DictIter::new(self)).into())
    }
}

impl DictLike for Set {
    fn borrow(&self) -> Ref<'_, InnerDict> {
        self.value.borrow()
    }
}
