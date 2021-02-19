use crate::custom_types::exceptions::key_error;
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_tools::next_power_2;
use crate::int_var::IntVar;
use crate::looping::{self, IterAttrs, IterResult, NativeIterator};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use crate::{first, first_two};
use std::cell::{Cell, RefCell};
use std::iter::Iterator;
use std::mem::{replace, take};
use std::rc::Rc;

#[derive(Debug, Clone)]
struct Entry {
    key: Variable,
    value: Variable,
    hash: usize,
    next: Option<Box<Entry>>,
}

#[derive(Debug)]
pub struct Dict {
    value: RefCell<InnerDict>,
}

#[derive(Debug)]
struct InnerDict {
    size: usize,
    entries: Vec<Option<Entry>>,
}

impl Dict {
    pub fn from_args(
        keys: Vec<Variable>,
        values: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> Result<Rc<Dict>, ()> {
        Result::Ok(Rc::new(Dict {
            value: RefCell::new(InnerDict::from_args(keys, values, runtime)?),
        }))
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
            _ => unimplemented!(),
        }
    }

    fn attr_fn(s: &str) -> NativeMethod<Rc<Dict>> {
        match s {
            "clear" => Dict::clear,
            "get" => Dict::get,
            "replace" => Dict::replace,
            "remove" => Dict::remove,
            "setDefault" => Dict::set_default,
            _ => unimplemented!(),
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
        let (key, val) = first_two(args);
        self.value.borrow_mut().set(key, val, runtime)
    }

    fn contains(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let is_in = self.value.borrow().get(first(args), runtime)?.is_some();
        runtime.return_1(is_in.into())
    }

    fn del(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        self.value.borrow_mut().del(&args[0], runtime)?;
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
            let (key, default) = first_two(args);
            let val = self.value.borrow().get(key, runtime)?.unwrap_or(default);
            runtime.return_1(val)
        }
    }

    fn replace(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let (key, val) = first_two(args);
        match self.value.borrow_mut().get_mut_entry(key, runtime)? {
            Option::Some(entry) => {
                let old = replace(&mut entry.value, val);
                runtime.return_1(Option::Some(old).into())
            }
            Option::None => runtime.return_1(Option::None.into()),
        }
    }

    fn remove(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let returned = self.value.borrow_mut().del(&args[0], runtime)?.into();
        runtime.return_1(returned)
    }

    fn set_default(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let mut value = self.value.borrow_mut();
        let (arg, default) = first_two(args);
        match value.get(arg.clone(), runtime)? {
            Option::Some(x) => runtime.return_1(x),
            Option::None => {
                value.set(arg, default.clone(), runtime)?;
                runtime.return_1(default)
            }
        }
    }

    fn eq(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            match downcast_var::<Dict>(arg) {
                Option::None => return runtime.return_1(false.into()),
                Option::Some(other) => {
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
        debug_assert!(args.is_empty()); // TODO: List of a value
        let dict = Dict::from_args(Vec::new(), Vec::new(), runtime)?;
        runtime.return_1(dict.into())
    }

    pub fn dict_type() -> Type {
        custom_class!(Dict, create, "dict")
    }

    fn is_empty(&self) -> bool {
        self.value.borrow().is_empty()
    }

    fn len(&self) -> usize {
        self.value.borrow().size
    }
}

impl InnerDict {
    pub fn from_args(
        keys: Vec<Variable>,
        values: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> Result<InnerDict, ()> {
        debug_assert!(keys.len() == values.len());
        let vec_capacity = next_power_2(keys.len());
        let mut value = InnerDict {
            size: 0,
            entries: vec![Option::None; vec_capacity],
        };
        for (x, y) in keys.into_iter().zip(values) {
            value.set(x, y, runtime)?;
        }
        Result::Ok(value)
    }

    pub fn get(&self, key: Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        let hash = key.clone().hash(runtime)?;
        if self.entries.is_empty() {
            Result::Ok(Option::None)
        } else {
            match &self.entries[hash % self.entries.len()] {
                Option::None => Result::Ok(Option::None),
                Option::Some(e) => e.get(key, hash, runtime),
            }
        }
    }

    fn true_repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        if self.is_empty() {
            return Result::Ok("{:}".into());
        }
        let mut result = String::new();
        result += "{";
        for (x, y) in self {
            result += x.clone().str(runtime)?.as_str();
            result += ": ";
            result += y.clone().str(runtime)?.as_str();
            result += ", ";
        }
        result.pop();
        result.pop();
        result += "}";
        Result::Ok(result.into())
    }

    pub fn set(&mut self, key: Variable, val: Variable, runtime: &mut Runtime) -> FnResult {
        let hash = key.clone().hash(runtime)?;
        self.resize(next_power_2(self.size + 1), runtime)?;
        let len = self.entries.len();
        match &mut self.entries[hash % len] {
            e @ Option::None => {
                e.replace(Entry {
                    key,
                    value: val,
                    hash,
                    next: None,
                });
                runtime.return_0()
            }
            Option::Some(e) => {
                let val = e.set(key, hash, val, runtime).ok_or(())?;
                if val {
                    self.size += 1;
                }
                runtime.return_0()
            }
        }
    }

    fn resize(&mut self, new_size: usize, runtime: &mut Runtime) -> FnResult {
        let current_size = self.entries.len();
        if current_size >= new_size {
            return FnResult::Ok(());
        }
        let old_vec = replace(&mut self.entries, vec![Option::None; new_size]);
        for entry in old_vec {
            if let Option::Some(mut e) = entry {
                loop {
                    let next = e.next.take();
                    self.set(e.key, e.value, runtime)?;
                    if let Option::Some(x) = next {
                        e = *x;
                    } else {
                        break;
                    }
                }
            }
        }
        FnResult::Ok(())
    }

    pub fn equals(&self, other: &InnerDict, runtime: &mut Runtime) -> Result<bool, ()> {
        if self.size != other.size {
            return Result::Ok(false);
        }
        for (key, value) in self {
            if !match other.get(key.clone(), runtime)? {
                Option::Some(val) => val.equals(value.clone(), runtime)?,
                Option::None => false,
            } {
                return Result::Ok(false);
            }
        }
        Result::Ok(true)
    }

    pub fn del(&mut self, value: &Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        let hash = value.clone().hash(runtime)?;
        let index = hash % self.entries.len();
        match &mut self.entries[index] {
            Option::Some(val) => match val.del(value, hash, runtime)? {
                Option::Some(result) => {
                    let boxed_entry = val.next.take();
                    self.entries[index] = boxed_entry.map(|x| *x);
                    Result::Ok(Option::Some(result))
                }
                Option::None => Result::Ok(Option::None),
            },
            Option::None => Result::Ok(Option::None),
        }
    }

    pub fn get_mut_entry(
        &mut self,
        key: Variable,
        runtime: &mut Runtime,
    ) -> Result<Option<&mut Entry>, ()> {
        let hash = key.clone().hash(runtime)?;
        let bucket = hash % self.entries.len();
        match &mut self.entries[bucket] {
            Option::Some(val) => val.get_mut_entry(key, hash, runtime),
            Option::None => Result::Ok(Option::None),
        }
    }

    pub fn clear(&mut self) {
        self.size = 0;
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl Entry {
    pub fn get(
        &self,
        key: Variable,
        hash: usize,
        runtime: &mut Runtime,
    ) -> Result<Option<Variable>, ()> {
        if self.hash == hash && key.clone().equals(self.key.clone(), runtime)? {
            Result::Ok(Option::Some(self.value.clone()))
        } else {
            match &self.next {
                Option::None => Result::Ok(Option::None),
                Option::Some(e) => e.get(key, hash, runtime),
            }
        }
    }

    pub fn set(
        &mut self,
        key: Variable,
        hash: usize,
        val: Variable,
        runtime: &mut Runtime,
    ) -> Option<bool> {
        if self.hash == hash && key.clone().equals(self.key.clone(), runtime).ok()? {
            self.value = val;
            Option::Some(false)
        } else {
            match &mut self.next {
                Option::None => {
                    self.next = Option::Some(Box::new(Entry {
                        key,
                        value: val,
                        hash,
                        next: Option::None,
                    }));
                    Option::Some(true)
                }
                Option::Some(e) => e.set(key, hash, val, runtime),
            }
        }
    }

    pub fn del(
        &mut self,
        key: &Variable,
        hash: usize,
        runtime: &mut Runtime,
    ) -> Result<Option<Variable>, ()> {
        if self.hash == hash && key.clone().equals(self.value.clone(), runtime)? {
            Result::Ok(Option::Some(take(&mut self.value)))
        } else {
            match &mut self.next {
                Option::None => Result::Ok(Option::None),
                Option::Some(e) => match e.del(key, hash, runtime)? {
                    Option::Some(val) => {
                        self.next = e.next.take();
                        Result::Ok(Option::Some(val))
                    }
                    Option::None => Result::Ok(Option::None),
                },
            }
        }
    }

    pub fn get_key(&self) -> &Variable {
        &self.key
    }

    pub fn get_value(&self) -> &Variable {
        &self.value
    }

    pub fn get_next(&self) -> &Option<Box<Entry>> {
        &self.next
    }

    fn get_entry(&self, key: Variable, runtime: &mut Runtime) -> Result<&Entry, ()> {
        if self.value.clone().equals(key.clone(), runtime)? {
            Result::Ok(self)
        } else {
            self.next
                .as_ref()
                .expect("Called get_entry() with no reasonable entry")
                .get_entry(key, runtime)
        }
    }

    fn get_mut_entry(
        &mut self,
        key: Variable,
        hash: usize,
        runtime: &mut Runtime,
    ) -> Result<Option<&mut Entry>, ()> {
        if self.hash == hash && self.value.clone().equals(key.clone(), runtime)? {
            Result::Ok(Option::Some(self))
        } else {
            match self.next.as_mut() {
                Option::Some(val) => val.get_mut_entry(key, hash, runtime),
                Option::None => Result::Ok(Option::None),
            }
        }
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
            "length" => return IntVar::from(self.len()).into(),
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
        Result::Ok(self.value.borrow().true_repr(runtime)?)
    }

    fn repr(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.value.borrow().true_repr(runtime)?)
    }

    fn bool(self: Rc<Self>, _runtime: &mut Runtime) -> Result<bool, ()> {
        Result::Ok(!self.value.borrow().is_empty())
    }

    fn iter(self: Rc<Self>, _runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        Result::Ok(Rc::new(DictIter::new(self)).into())
    }
}

impl<'a> IntoIterator for &'a InnerDict {
    type Item = (&'a Variable, &'a Variable);
    type IntoIter = InnerDictIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        for (i, value) in self.entries.iter().enumerate() {
            if let Option::Some(x) = value {
                return InnerDictIter {
                    parent: self,
                    i,
                    current: Option::Some(x),
                };
            }
        }
        InnerDictIter {
            parent: self,
            i: self.entries.len(),
            current: Option::None,
        }
    }
}

#[derive(Debug)]
struct DictIter {
    parent: Rc<Dict>,
    bucket_no: Cell<usize>,
    index: RefCell<Variable>,
}

impl DictIter {
    fn new(parent: Rc<Dict>) -> DictIter {
        let val = DictIter {
            parent,
            bucket_no: Cell::new(0),
            index: RefCell::new(Variable::null()),
        };
        val.point_to_next();
        val
    }

    fn point_to_next(&self) {
        let parent = self.parent.value.borrow();
        let mut next = self.bucket_no.get();
        while next < parent.size {
            if let Option::Some(val) = parent.entries[next].as_ref() {
                self.bucket_no.set(next);
                self.index.replace(val.get_key().clone());
                return;
            }
            next += 1;
        }
        self.bucket_no.set(next);
    }

    fn true_next(
        self: Rc<Self>,
        runtime: &mut Runtime,
    ) -> Result<Option<(Variable, Variable)>, ()> {
        let len = self.parent.len();
        let bucket = self.bucket_no.get();
        if bucket >= len {
            return Result::Ok(Option::None);
        }
        let parent = self.parent.value.borrow();
        let parent_node = parent.entries[bucket]
            .as_ref()
            .expect("Dict iterator expects self.bucket_no to always point at a non-None value");
        let node = parent_node.get_entry(self.index.borrow().clone(), runtime)?;
        let key = self.index.replace(Variable::null());
        let val = node.get_value().clone();
        debug_assert!(node.get_key().clone().equals(key.clone(), runtime)?);
        if let Option::Some(next) = node.get_next() {
            self.index.replace(next.get_value().clone());
        } else {
            self.point_to_next();
        }
        Result::Ok(Option::Some((key, val)))
    }
}

impl IterAttrs for DictIter {
    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.true_next(runtime)? {
            Option::None => runtime.return_n(vec![Option::None.into(), Option::None.into()]),
            Option::Some(val) => {
                runtime.return_n(vec![Option::Some(val.0).into(), Option::Some(val.1).into()])
            }
        }
    }

    fn get_type() -> Type {
        unimplemented!()
    }
}

impl NativeIterator for DictIter {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.true_next(runtime)?.map(|(a, b)| vec![a, b]).into())
    }
}

struct InnerDictIter<'a> {
    parent: &'a InnerDict,
    i: usize,
    current: Option<&'a Entry>,
}

impl InnerDictIter<'_> {
    fn adjust_i(&mut self) {
        self.i += 1;
        while self.i < self.parent.entries.len() && self.parent.entries[self.i].is_none() {
            self.i += 1;
        }
    }
}

impl<'a> Iterator for InnerDictIter<'a> {
    type Item = (&'a Variable, &'a Variable);

    fn next(&mut self) -> Option<Self::Item> {
        match take(&mut self.current) {
            Option::None => Option::None,
            Option::Some(entry) => {
                self.current = match &entry.next {
                    Option::Some(x) => Option::Some(&**x),
                    Option::None => {
                        self.adjust_i();
                        self.parent.entries[self.i].as_ref()
                    }
                };
                Option::Some((&entry.key, &entry.value))
            }
        }
    }
}
