use crate::custom_types::exceptions::key_error;
use crate::custom_types::ASCII_COMMA;
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_var::IntVar;
use crate::looping::{self, IterAttrs, IterResult, NativeIterator};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::{MaybeString, StringVar};
use crate::variable::{FnResult, Variable};
use crate::{first, first_n};
use ascii::{AsciiChar, AsciiStr};
use once_cell::sync::Lazy;
use std::cell::{Cell, Ref, RefCell};
use std::cmp::{max, min};
use std::fmt::Debug;
use std::iter::{FusedIterator, Iterator};
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

struct EntryMut<'a> {
    size: &'a mut usize,
    size_w_deleted: &'a mut usize,
    entry: &'a mut Entry,
}

#[derive(Debug)]
pub struct Dict {
    value: RefCell<InnerDict>,
}

#[derive(Debug, Clone)]
pub(super) struct InnerDict {
    size: usize,
    size_w_deleted: usize,
    entries: Vec<Entry>,
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

    fn replace(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let [key, val] = first_n(args);
        let mut value = self.value.borrow_mut();
        if let Result::Ok(e) = value.entry_mut(key, runtime)?.into_value() {
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
        let [arg, default] = first_n(args);
        value.resize(1);
        let result = match value.entry_mut(arg.clone(), runtime)?.into_value() {
            Result::Ok(e) => e.value.clone(),
            Result::Err(mut e) => {
                let hash = arg.clone().hash(runtime)?;
                e.put(arg, default.clone(), hash);
                default
            }
        };
        runtime.return_1(result)
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
        self.value.borrow().size
    }
}

impl InnerDict {
    const PERTURB_SHIFT: u32 = 5;
    const MIN_SIZE: usize = 8;

    pub fn new() -> InnerDict {
        InnerDict {
            size: 0,
            size_w_deleted: 0,
            entries: Vec::new(),
        }
    }

    pub fn from_args(
        keys: Vec<Variable>,
        values: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> Result<InnerDict, ()> {
        debug_assert!(keys.len() == values.len());
        if keys.is_empty() {
            Result::Ok(InnerDict::new())
        } else {
            let vec_capacity = Self::new_cap(0, keys.len());
            let mut value = InnerDict {
                size: 0,
                size_w_deleted: 0,
                entries: vec![Entry::None; vec_capacity],
            };
            for (x, y) in keys.into_iter().zip(values) {
                value.set(x, y, runtime)?;
            }
            Result::Ok(value)
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get(&self, key: Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        if self.entries.is_empty() {
            Result::Ok(Option::None)
        } else if let Entry::Some(e) = self.entry(key, runtime)? {
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
        self.resize(1);
        assert!(!self.entries.is_empty());
        Result::Ok(self.entry_mut(key.clone(), runtime)?.put(key, val, hash))
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
        Result::Ok(self.entry_mut(value, runtime)?.remove())
    }

    pub fn clear(&mut self) {
        self.size = 0;
        self.size_w_deleted = 0;
        for entry in &mut self.entries {
            entry.take();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn true_repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        static ASCII_COLON: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii(": ").unwrap());
        if self.is_empty() {
            static EMPTY_DICT: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii("{:}").unwrap());
            return Result::Ok((*EMPTY_DICT).into());
        }
        let mut result = MaybeString::new();
        result.push_ascii(AsciiChar::CurlyBraceOpen);
        let mut first = true;
        for entry in &self.entries {
            if let Entry::Some(e) = entry {
                if !first {
                    result += *ASCII_COMMA;
                }
                first = false;
                result += e.key.clone().str(runtime)?;
                result += *ASCII_COLON;
                result += e.value.clone().str(runtime)?;
            }
        }
        result.push_ascii(AsciiChar::CurlyBraceClose);
        Result::Ok(result.into())
    }

    pub fn key_repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        if self.is_empty() {
            static EMPTY_SET: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii("{}").unwrap());
            return Result::Ok((*EMPTY_SET).into());
        }
        let mut result = MaybeString::new();
        result.push_ascii(AsciiChar::CurlyBraceOpen);
        let mut first = true;
        for entry in &self.entries {
            if let Entry::Some(e) = entry {
                if !first {
                    result += *ASCII_COMMA;
                }
                first = false;
                result += e.key.clone().str(runtime)?;
            }
        }
        result.push_ascii(AsciiChar::CurlyBraceClose);
        Result::Ok(result.into())
    }

    fn entry(&self, key: Variable, runtime: &mut Runtime) -> Result<&Entry, ()> {
        assert!(!self.entries.is_empty());
        let len = self.entries.len();
        let hash = key.clone().hash(runtime)?;
        let mut perturb = hash;
        let mut bucket = hash % len;
        let mut first_removed = Option::None;
        loop {
            match &self.entries[bucket] {
                e @ Entry::None => return Result::Ok(first_removed.unwrap_or(e)),
                e @ Entry::Removed => {
                    first_removed.get_or_insert(e);
                    bucket = Self::rehash(&mut perturb, bucket) % len;
                }
                Entry::Some(e) => {
                    if e.hash == hash && e.key.clone().equals(key.clone(), runtime)? {
                        return Result::Ok(&self.entries[bucket]);
                    }
                    bucket = Self::rehash(&mut perturb, bucket) % len;
                }
            }
        }
    }

    fn entry_mut(&mut self, key: Variable, runtime: &mut Runtime) -> Result<EntryMut<'_>, ()> {
        assert!(!self.entries.is_empty());
        let len = self.entries.len();
        let hash = key.clone().hash(runtime)?;
        let mut perturb = hash;
        let mut bucket = hash % len;
        let mut first_removed = Option::None;
        let bucket: usize = loop {
            match &mut self.entries[bucket] {
                Entry::None => {
                    break first_removed.unwrap_or(bucket);
                }
                Entry::Removed => {
                    first_removed.get_or_insert(bucket);
                    bucket = Self::rehash(&mut perturb, bucket) % len;
                }
                Entry::Some(e) => {
                    if e.hash == hash && e.key.clone().equals(key.clone(), runtime)? {
                        break bucket;
                    } else {
                        bucket = Self::rehash(&mut perturb, bucket) % len;
                    }
                }
            }
        };
        Result::Ok(EntryMut {
            size: &mut self.size,
            size_w_deleted: &mut self.size_w_deleted,
            entry: &mut self.entries[bucket],
        })
    }

    fn resize(&mut self, additional: usize) {
        let new_size = self.size_w_deleted + additional;
        let new_capacity = self.new_capacity(new_size);
        let current_size = self.entries.len();
        if current_size < new_capacity {
            // Resize ignoring the deleted values b/c they'll all disappear in resizing
            let new_cap = max(current_size, self.new_capacity(self.size + additional));
            self.resize_exact(new_cap);
        }
    }

    fn resize_exact(&mut self, new_size: usize) {
        debug_assert!(new_size.is_power_of_two());
        let old_vec = replace(&mut self.entries, vec![Entry::None; new_size]);
        let new_vec = &mut self.entries;
        let len = new_vec.len();
        for entry in old_vec {
            // Doesn't use self.set here b/c we already know all elements are unique, and we know
            // the hash of each element already
            if let Entry::Some(entry) = entry {
                let hash = entry.hash;
                let mut bucket = hash % len;
                let mut perturb = hash;
                while let Entry::Some(_) = new_vec[bucket] {
                    bucket = Self::rehash(&mut perturb, bucket) % len;
                }
                new_vec[bucket] = Entry::Some(entry);
            }
        }
        self.size_w_deleted = self.size;
    }

    fn new_capacity(&self, new_size: usize) -> usize {
        Self::new_cap(self.entries.len(), new_size)
    }

    fn new_cap(current_cap: usize, new_size: usize) -> usize {
        const LOAD_FACTOR: f64 = 0.75;
        if current_cap as f64 * LOAD_FACTOR >= new_size as f64 {
            return current_cap;
        }
        let mut new_cap = max(Self::MIN_SIZE, new_size.next_power_of_two());
        while new_cap as f64 * LOAD_FACTOR < new_size as f64 {
            new_cap <<= 1;
        }
        new_cap
    }

    fn rehash(perturb: &mut usize, bucket: usize) -> usize {
        let result = 5 * bucket + 1 + *perturb;
        *perturb >>= Self::PERTURB_SHIFT;
        result
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

impl<'a> EntryMut<'a> {
    pub fn remove(&mut self) -> Option<Variable> {
        match &mut self.entry {
            Entry::None => Option::None,
            Entry::Removed => Option::None,
            e @ Entry::Some(_) => {
                let entry = e.remove().unwrap();
                *self.size -= 1;
                Option::Some(entry.value)
            }
        }
    }

    pub fn put(&mut self, key: Variable, val: Variable, hash: usize) -> Option<Variable> {
        match &mut self.entry {
            e @ Entry::None | e @ Entry::Removed => {
                let old_entry = replace(
                    *e,
                    Entry::Some(InnerEntry {
                        key,
                        hash,
                        value: val,
                    }),
                );
                if let Entry::None = old_entry {
                    *self.size_w_deleted += 1;
                }
                *self.size += 1;
                Option::None
            }
            Entry::Some(e) => Option::Some(replace(&mut e.value, val)),
        }
    }

    pub fn into_value(self) -> Result<&'a mut InnerEntry, Self> {
        match self.entry {
            Entry::Some(e) => Result::Ok(e),
            _ => Result::Err(self),
        }
    }
}

impl<T: DictLike + 'static> IterAttrs for DictIter<T> {
    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.true_next() {
            Option::None => runtime.return_n([Option::None.into(), Option::None.into()]),
            Option::Some(val) => {
                runtime.return_n([Option::Some(val.0).into(), Option::Some(val.1).into()])
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min_max = min(self.parent.size, self.parent.entries.len() - self.i);
        (0, Option::Some(min_max))
    }
}

impl FusedIterator for InnerDictIter<'_> {}
