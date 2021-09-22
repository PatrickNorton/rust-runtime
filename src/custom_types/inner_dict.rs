use crate::custom_types::ASCII_COMMA;
use crate::looping::{IterAttrs, IterResult, NativeIterator};
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::{MaybeString, StringVar};
use crate::variable::{FnResult, Variable};
use ascii::{AsciiChar, AsciiStr};
use once_cell::sync::Lazy;
use std::cell::{Cell, Ref};
use std::cmp::{max, min};
use std::fmt::Debug;
use std::iter::FusedIterator;
use std::mem::{replace, take};
use std::rc::Rc;

pub(super) trait DictLike: Debug {
    fn borrow(&self) -> Ref<'_, InnerDict>;
}

#[derive(Debug, Clone)]
pub(super) struct InnerDict {
    size: usize,
    size_w_deleted: usize,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub(super) enum Entry {
    None,
    Removed,
    Some(InnerEntry),
}

#[derive(Debug, Clone)]
pub(super) struct InnerEntry {
    key: Variable,
    value: Variable,
    hash: usize,
}

pub(super) struct EntryMut<'a> {
    size: &'a mut usize,
    size_w_deleted: &'a mut usize,
    entry: &'a mut Entry,
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

    pub(super) fn entries_raw_mut(&mut self) -> &mut [Entry] {
        &mut self.entries
    }

    pub(super) fn size_mut(&mut self) -> &mut usize {
        &mut self.size
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

    pub fn get_pair(
        &self,
        key: Variable,
        runtime: &mut Runtime,
    ) -> Result<Option<(Variable, Variable)>, ()> {
        if self.entries.is_empty() {
            Result::Ok(Option::None)
        } else if let Entry::Some(e) = self.entry(key, runtime)? {
            Result::Ok(Option::Some((e.key.clone(), e.value.clone())))
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

    pub(crate) fn entry_mut(
        &mut self,
        key: Variable,
        runtime: &mut Runtime,
    ) -> Result<EntryMut<'_>, ()> {
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

    pub(super) fn resize(&mut self, additional: usize) {
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

    fn requires_resize(current_cap: usize, new_size: usize) -> bool {
        // Equivalent to load factor of 0.75, but without loss of precision from floats
        current_cap - (current_cap / 4) < new_size
    }

    fn new_cap(current_cap: usize, new_size: usize) -> usize {
        if !Self::requires_resize(current_cap, new_size) {
            return current_cap;
        }
        let new_cap = max(Self::MIN_SIZE, new_size.next_power_of_two());
        if Self::requires_resize(current_cap, new_cap) {
            new_cap << 1
        } else {
            new_cap
        }
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

impl InnerEntry {
    pub fn clone_key(&self) -> Variable {
        self.key.clone()
    }

    pub fn clone_value(&self) -> Variable {
        self.value.clone()
    }

    pub fn value_mut(&mut self) -> &mut Variable {
        &mut self.value
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
        let len = parent.entries.len();
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

impl<'a> IntoIterator for &'a InnerDict {
    type Item = (&'a Variable, &'a Variable);
    type IntoIter = InnerDictIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        InnerDictIter { parent: self, i: 0 }
    }
}

impl FusedIterator for InnerDictIter<'_> {}
