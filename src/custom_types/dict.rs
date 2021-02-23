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
use std::cell::{Cell, Ref, RefCell};
use std::fmt::Debug;
use std::iter::Iterator;
use std::mem::{replace, take};
use std::rc::Rc;

pub(super) trait DictLike: Debug {
    fn borrow(&self) -> Ref<'_, InnerDict>;
}

#[derive(Debug, Clone)]
enum Entry {
    None,
    Removed,
    Some(InnerEntry),
}

#[derive(Debug, Clone)]
struct InnerEntry {
    key: Variable,
    value: Variable,
    hash: usize,
}

#[derive(Debug)]
pub struct Dict {
    value: RefCell<InnerDict>,
}

#[derive(Debug, Clone)]
pub(super) struct InnerDict {
    size: usize,
    entries: Vec<Entry>,
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
            _ => unimplemented!("dict.{}", o.name()),
        }
    }

    fn attr_fn(s: &str) -> NativeMethod<Rc<Dict>> {
        match s {
            "clear" => Dict::clear,
            "get" => Dict::get,
            "replace" => Dict::replace,
            "remove" => Dict::remove,
            "setDefault" => Dict::set_default,
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
        let (key, val) = first_two(args);
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
            let (key, default) = first_two(args);
            let val = self.value.borrow().get(key, runtime)?.unwrap_or(default);
            runtime.return_1(val)
        }
    }

    fn replace(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let (key, val) = first_two(args);
        if let Option::Some(Entry::Some(e)) = self.value.borrow_mut().entry_mut(key, runtime)? {
            runtime.return_1(Option::Some(replace(&mut e.value, val)).into())
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
        let (arg, default) = first_two(args);
        let new_size = value.size + 1;
        value.resize(new_size, runtime)?;
        match value.entry_mut(arg.clone(), runtime)? {
            Option::Some(entry) => match entry {
                e @ Entry::None | e @ Entry::Removed => {
                    let hash = arg.clone().hash(runtime)?;
                    *e = Entry::Some(InnerEntry {
                        key: arg,
                        value: default.clone(),
                        hash,
                    });
                    runtime.return_1(default)
                }
                Entry::Some(e) => runtime.return_1(e.value.clone()),
            },
            Option::None => {
                value
                    .set(arg, default.clone(), runtime)
                    .expect_err("Value.entry_mut should have returned Option::Some");
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
            entries: vec![Entry::None; vec_capacity],
        };
        for (x, y) in keys.into_iter().zip(values) {
            value.set(x, y, runtime)?;
        }
        Result::Ok(value)
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get(&self, key: Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        if self.entries.is_empty() {
            Result::Ok(Option::None)
        } else if let Option::Some(Entry::Some(e)) = self.entry(key, runtime)? {
            Result::Ok(Option::Some(e.value.clone()))
        } else {
            Result::Ok(Option::None)
        }
    }

    pub fn set(
        &mut self,
        key: Variable,
        val: Variable,
        runtime: &mut Runtime,
    ) -> Result<Option<Variable>, ()> {
        let hash = key.clone().hash(runtime)?;
        self.resize(self.size + 1, runtime)?;
        assert!(!self.entries.is_empty());
        match self.entry_mut(key.clone(), runtime)? {
            Option::Some(entry) => match entry {
                e @ Entry::None | e @ Entry::Removed => {
                    *e = Entry::Some(InnerEntry {
                        key,
                        hash,
                        value: val,
                    });
                    self.size += 1;
                    Result::Ok(Option::None)
                }
                Entry::Some(e) => Result::Ok(Option::Some(replace(&mut e.value, val))),
            },
            Option::None => panic!("No suitable entry found, {:?}", self),
        }
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

    pub fn del(&mut self, value: Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        match self.entry_mut(value, runtime)? {
            Option::None => Result::Ok(Option::None),
            Option::Some(Entry::None) => Result::Ok(Option::None),
            Option::Some(Entry::Removed) => Result::Ok(Option::None),
            Option::Some(e @ Entry::Some(_)) => {
                let entry = e.remove().unwrap();
                Result::Ok(Option::Some(entry.value))
            }
        }
    }

    pub fn clear(&mut self) {
        self.size = 0;
        for entry in &mut self.entries {
            entry.take();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn true_repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        if self.is_empty() {
            return Result::Ok("{:}".into());
        }
        let mut result = String::new();
        result += "{";
        for entry in &self.entries {
            if let Entry::Some(e) = entry {
                result += e.key.clone().str(runtime)?.as_str();
                result += ": ";
                result += e.key.clone().str(runtime)?.as_str();
                result += ", ";
            }
        }
        result.pop();
        result.pop();
        result += "}";
        Result::Ok(result.into())
    }
    
    pub fn key_repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        if self.is_empty() {
            return Result::Ok("{}".into());
        }
        let mut result = String::new();
        result += "{";
        for entry in &self.entries {
            if let Entry::Some(e) = entry {
                result += e.key.clone().str(runtime)?.as_str();
                result += ", ";
            }
        }
        result.pop();
        result.pop();
        result += "}";
        Result::Ok(result.into())
    }

    fn entry(&self, key: Variable, runtime: &mut Runtime) -> Result<Option<&Entry>, ()> {
        // Returning Option::None means that no entry was found, but neither was a suitable
        // empty value (all given were full)
        assert!(!self.entries.is_empty());
        let len = self.entries.len();
        let hash = key.clone().hash(runtime)?;
        let mut bucket = hash % len;
        let mut first_removed = Option::None;
        loop {
            match &self.entries[bucket] {
                e @ Entry::None => return Result::Ok(Option::Some(first_removed.unwrap_or(e))),
                e @ Entry::Removed => {
                    let rehash = Self::rehash(hash, bucket) % len;
                    if rehash == hash % len {
                        return Result::Ok(Option::Some(first_removed.unwrap_or(e)));
                    } else {
                        first_removed.get_or_insert(e);
                        bucket = rehash;
                    }
                }
                Entry::Some(e) => {
                    if e.hash == hash && e.key.clone().equals(key.clone(), runtime)? {
                        return Result::Ok(Option::Some(&self.entries[bucket]));
                    }
                    let rehash = Self::rehash(hash, bucket) % len;
                    if rehash == hash % len {
                        return Result::Ok(first_removed);
                    } else {
                        bucket = rehash
                    }
                }
            }
        }
    }

    fn entry_mut(
        &mut self,
        key: Variable,
        runtime: &mut Runtime,
    ) -> Result<Option<&mut Entry>, ()> {
        assert!(!self.entries.is_empty());
        let len = self.entries.len();
        let hash = key.clone().hash(runtime)?;
        let mut bucket = hash % len;
        let mut first_removed = Option::None;
        let bucket: usize = loop {
            match &mut self.entries[bucket] {
                Entry::None => {
                    break first_removed.unwrap_or(bucket);
                }
                Entry::Removed => {
                    let rehash = Self::rehash(hash, bucket) % len;
                    if rehash == hash % len {
                        break first_removed.unwrap_or(bucket);
                    } else {
                        first_removed.get_or_insert(bucket);
                        bucket = rehash;
                    }
                }
                Entry::Some(e) => {
                    if e.hash == hash && e.key.clone().equals(key.clone(), runtime)? {
                        break bucket;
                    } else {
                        let rehash = Self::rehash(hash, bucket) % len;
                        if rehash == hash % len {
                            match first_removed {
                                Option::None => return Result::Ok(Option::None),
                                Option::Some(rem) => break rem,
                            }
                        } else {
                            bucket = rehash;
                        }
                    }
                }
            }
        };
        Result::Ok(Option::Some(&mut self.entries[bucket]))
    }

    fn resize(&mut self, new_size: usize, runtime: &mut Runtime) -> FnResult {
        const LOAD_FACTOR: f64 = 0.75;
        let current_size = self.entries.len();
        if current_size as f64 * LOAD_FACTOR >= new_size as f64 {
            return FnResult::Ok(());
        }
        let old_vec = replace(&mut self.entries, vec![Entry::None; next_power_2(new_size)]);
        for entry in old_vec {
            if let Entry::Some(e) = entry {
                self.set(e.key, e.value, runtime)?;
            }
        }
        FnResult::Ok(())
    }

    fn rehash(hash: usize, bucket: usize) -> usize {
        const PERTURB_SHIFT: u32 = 5;
        5 * bucket + 1 + (hash.wrapping_shr(PERTURB_SHIFT))
    }
}

impl Entry {
    pub fn take(&mut self) -> Self {
        take(self)
    }

    pub fn remove(&mut self) -> Self {
        replace(self, Entry::Removed)
    }

    pub fn unwrap(self) -> InnerEntry {
        match self {
            Entry::None => panic!(),
            Entry::Removed => panic!(),
            Entry::Some(e) => e,
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Entry::None
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

impl DictLike for Dict {
    fn borrow(&self) -> Ref<'_, InnerDict> {
        unimplemented!()
    }
}

impl<'a> IntoIterator for &'a InnerDict {
    type Item = (&'a Variable, &'a Variable);
    type IntoIter = InnerDictIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        InnerDictIter { parent: self, i: 0 }
    }
}

#[derive(Debug)]
pub(super) struct DictIter<T: DictLike> {
    parent: Rc<T>,
    bucket_no: Cell<usize>,
}

impl<T: DictLike> DictIter<T> {
    pub fn new(parent: Rc<T>) -> DictIter<T> {
        DictIter {
            parent,
            bucket_no: Cell::new(0),
        }
    }

    fn true_next(self: Rc<Self>) -> Option<(Variable, Variable)> {
        let parent = self.parent.borrow();
        let len = parent.size;
        let mut bucket = self.bucket_no.get();
        if bucket >= len {
            return Option::None;
        }
        loop {
            if bucket >= len {
                self.bucket_no.set(bucket);
                return Option::None;
            } else if let Entry::Some(e) = &parent.entries[bucket] {
                bucket += 1;
                self.bucket_no.set(bucket);
                return Option::Some((e.key.clone(), e.value.clone()));
            } else {
                bucket += 1;
            }
        }
    }
}

impl<T: DictLike + 'static> IterAttrs for DictIter<T> {
    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.true_next() {
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

impl<T: DictLike + 'static> NativeIterator for DictIter<T> {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.true_next().map(|(a, b)| vec![a, b]).into())
    }
}

pub(super) struct InnerDictIter<'a> {
    parent: &'a InnerDict,
    i: usize,
}

impl<'a> Iterator for InnerDictIter<'a> {
    type Item = (&'a Variable, &'a Variable);

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.parent.entries.len();
        loop {
            if self.i >= len {
                return Option::None;
            } else if let Entry::Some(e) = &self.parent.entries[self.i] {
                self.i += 1;
                return Option::Some((&e.key, &e.value));
            } else {
                self.i += 1;
            }
        }
    }
}
