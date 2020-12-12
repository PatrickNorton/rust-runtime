use crate::custom_types::exceptions::key_error;
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_tools::next_power_2;
use crate::looping::{IterResult, NativeIterator};
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
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

    fn get_op(self: &Rc<Self>, o: Operator) -> Variable {
        let func = match o {
            Operator::GetAttr => Dict::index,
            Operator::Repr => Dict::repr,
            Operator::Str => Dict::repr,
            Operator::Bool => Dict::bool,
            Operator::SetAttr => Dict::set,
            Operator::In => Dict::contains,
            Operator::Equals => Dict::eq,
            Operator::Iter => Dict::iter,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn get_attribute(self: &Rc<Self>, s: StringVar) -> Variable {
        let func = match s.as_str() {
            "clear" => Dict::clear,
            "get" => Dict::get,
            "replace" => Dict::replace,
            "pop" => Dict::pop,
            "setDefault" => Dict::set_default,
            "length" => return Variable::Bigint(self.len().into()),
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn index(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        match self.value.borrow().get(args.remove(0), runtime)? {
            Option::Some(result) => runtime.return_1(result),
            Option::None => runtime.throw_quick(key_error(), "Value not found".into()),
        }
    }

    fn repr(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let repr = self.value.borrow().true_repr(runtime)?;
        runtime.return_1(repr.into())
    }

    fn bool(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.is_empty()).into())
    }

    fn set(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let val = args.remove(1); // Reverse order to avoid move
        let key = args.remove(0);
        self.value.borrow_mut().set(key, val, runtime)
    }

    fn contains(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = args.remove(0);
        let is_in = self.value.borrow().get(val, runtime)?.is_some();
        runtime.return_1(is_in.into())
    }

    fn clear(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().clear();
        runtime.return_0()
    }

    fn get(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        if args.len() == 1 {
            let val = self.value.borrow().get(take(&mut args[0]), runtime)?.into();
            runtime.return_1(val)
        } else {
            debug_assert_eq!(args.len(), 2);
            let val = self
                .value
                .borrow()
                .get(take(&mut args[0]), runtime)?
                .unwrap_or_else(|| take(&mut args[1]));
            runtime.return_1(val)
        }
    }

    fn replace(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let key = replace(&mut args[0], Variable::Null());
        let val = replace(&mut args[1], Variable::Null());
        match self.value.borrow_mut().get_mut_entry(key, runtime)? {
            Option::Some(entry) => {
                let old = replace(&mut entry.value, val);
                runtime.return_1(Option::Some(old).into())
            }
            Option::None => runtime.return_1(Option::None.into()),
        }
    }

    fn pop(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let returned = self
            .value
            .borrow_mut()
            .del(&args[0], runtime)?
            .unwrap_or_else(Default::default);
        runtime.return_1(returned)
    }

    fn set_default(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let mut value = self.value.borrow_mut();
        let arg = take(&mut args[0]);
        match value.get(arg.clone(), runtime)? {
            Option::Some(x) => runtime.return_1(x),
            Option::None => {
                let val = take(&mut args[1]);
                value.set(arg, val.clone(), runtime)?;
                runtime.return_1(val)
            }
        }
    }

    fn eq(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
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

    fn iter(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(DictIter::new(self.clone())).into())
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
        let hash = key.hash(runtime)?;
        match &self.entries[hash % self.entries.len()] {
            Option::None => Result::Err(()),
            Option::Some(e) => e.get(key, runtime),
        }
    }

    fn true_repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        if self.is_empty() {
            return Result::Ok("{:}".into());
        }
        let mut result = String::new();
        result += "{";
        self.for_each(|x, y| {
            result += x.clone().str(runtime)?.as_str();
            result += ": ";
            result += y.clone().str(runtime)?.as_str();
            result += ", ";
            FnResult::Ok(())
        })?;
        result.remove(result.len() - 1);
        result.remove(result.len() - 1);
        result += "}";
        Result::Ok(result.into())
    }

    fn for_each(&self, mut func: impl FnMut(&Variable, &Variable) -> FnResult) -> FnResult {
        for val in &self.entries {
            if let Option::Some(o) = val {
                func(o.get_key(), o.get_value())?;
                let mut p = o.get_next().as_ref();
                while let Option::Some(q) = p {
                    func(o.get_key(), o.get_value())?;
                    p = q.get_next().as_ref()
                }
            }
        }
        FnResult::Ok(())
    }

    pub fn set(&mut self, key: Variable, val: Variable, runtime: &mut Runtime) -> FnResult {
        let hash = key.hash(runtime)?;
        let len = self.entries.len();
        self.resize(next_power_2(self.size + 1), runtime)?;
        match &mut self.entries[hash % len] {
            Option::None => Result::Err(()),
            Option::Some(e) => {
                let val = e.set(key, val, runtime).ok_or(())?;
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
                    let (entry, next) = Self::split_entries(e);
                    self.set(entry.key, entry.value, runtime)?;
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
        for val in &self.entries {
            if let Option::Some(o) = val {
                if !Self::contains_and_eq(o, other, runtime)? {
                    return Result::Ok(false);
                }
                let mut p = o.get_next().as_ref();
                while let Option::Some(q) = p {
                    if !Self::contains_and_eq(q.as_ref(), other, runtime)? {
                        return Result::Ok(false);
                    }
                    p = q.get_next().as_ref()
                }
            }
        }
        Result::Ok(true)
    }

    pub fn del(&mut self, value: &Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        let hash = value.hash(runtime)?;
        let index = hash % self.entries.len();
        match &mut self.entries[index] {
            Option::Some(val) => match val.del(value, runtime)? {
                Option::Some(result) => {
                    let boxed_entry = replace(&mut val.next, Option::None);
                    self.entries[index] = boxed_entry.map(|x| *x);
                    Result::Ok(Option::Some(result))
                }
                Option::None => Result::Ok(Option::None),
            },
            Option::None => Result::Ok(Option::None),
        }
    }

    fn contains_and_eq(
        entry: &Entry,
        other: &InnerDict,
        runtime: &mut Runtime,
    ) -> Result<bool, ()> {
        match other.get(entry.key.clone(), runtime)? {
            Option::Some(val) => val.equals(entry.value.clone(), runtime),
            Option::None => Result::Ok(false),
        }
    }

    fn split_entries(mut e: Entry) -> (Entry, Option<Box<Entry>>) {
        let next = replace(&mut e.next, Option::None);
        (e, next)
    }

    pub fn get_mut_entry(
        &mut self,
        key: Variable,
        runtime: &mut Runtime,
    ) -> Result<Option<&mut Entry>, ()> {
        let hash = key.hash(runtime)?;
        let bucket = hash % self.entries.len();
        match &mut self.entries[bucket] {
            Option::Some(val) => val.get_mut_entry(key, runtime),
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
    pub fn get(&self, key: Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        if key.equals(self.value.clone(), runtime)? {
            Result::Ok(Option::Some(self.value.clone()))
        } else {
            match &self.next {
                Option::None => Result::Ok(Option::None),
                Option::Some(e) => e.get(key, runtime),
            }
        }
    }

    pub fn set(&mut self, key: Variable, val: Variable, runtime: &mut Runtime) -> Option<bool> {
        if key.equals(self.value.clone(), runtime).ok()? {
            self.value = val;
            Option::Some(false)
        } else {
            match &mut self.next {
                Option::None => {
                    let hash = key.hash(runtime).ok()?;
                    self.next = Option::Some(Box::new(Entry {
                        key,
                        value: val,
                        hash,
                        next: Option::None,
                    }));
                    Option::Some(true)
                }
                Option::Some(e) => e.set(key, val, runtime),
            }
        }
    }

    pub fn del(&mut self, key: &Variable, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        if key.equals(self.value.clone(), runtime)? {
            Result::Ok(Option::Some(replace(&mut self.value, Variable::Null())))
        } else {
            match &mut self.next {
                Option::None => Result::Ok(Option::None),
                Option::Some(e) => match e.del(key, runtime)? {
                    Option::Some(val) => {
                        self.next = replace(&mut e.next, Option::None);
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
        if self.value.equals(key.clone(), runtime)? {
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
        runtime: &mut Runtime,
    ) -> Result<Option<&mut Entry>, ()> {
        if self.value.equals(key.clone(), runtime)? {
            Result::Ok(Option::Some(self))
        } else {
            match self.next.as_mut() {
                Option::Some(val) => val.get_mut_entry(key, runtime),
                Option::None => Result::Ok(Option::None),
            }
        }
    }
}

impl CustomVar for Dict {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        name.do_each(|o| self.get_op(o), |s| self.get_attribute(s))
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Dict::dict_type()
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
            index: RefCell::new(Variable::Null()),
        };
        val.point_to_next();
        val
    }

    fn point_to_next(&self) {
        let parent = self.parent.value.borrow();
        while self.bucket_no.get() < parent.size {
            if let Option::Some(val) = parent.entries[self.bucket_no.get()].as_ref() {
                self.index.replace(val.get_key().clone());
                return;
            }
            self.bucket_no.set(self.bucket_no.get() + 1);
        }
    }

    fn next_fn(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.clone().true_next(runtime)? {
            Option::None => runtime.return_n(vec![Option::None.into(), Option::None.into()]),
            Option::Some(val) => {
                runtime.return_n(vec![Option::Some(val.0).into(), Option::Some(val.1).into()])
            }
        }
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
        let key = self.index.replace(Variable::Null());
        let val = node.get_value().clone();
        debug_assert!(node.get_key().equals(key.clone(), runtime)?);
        if let Option::Some(next) = node.get_next() {
            self.index.replace(next.get_value().clone());
        } else {
            self.point_to_next();
        }
        Result::Ok(Option::Some((key, val)))
    }
}

impl CustomVar for DictIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        let func = match name {
            Name::Operator(_) => unimplemented!(),
            Name::Attribute(val) => match val.as_str() {
                "next" => Self::next_fn,
                _ => unimplemented!(),
            },
        };
        Variable::Method(StdMethod::new_native(self, func))
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        unimplemented!()
    }
}

impl NativeIterator for DictIter {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.true_next(runtime)?.map(|f| f.0))
    }
}
