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
use std::mem::replace;
use std::rc::Rc;

#[derive(Debug)]
pub struct Set {
    value: RefCell<InnerSet>,
}

#[derive(Debug)]
struct InnerSet {
    size: usize,
    values: Vec<Option<Entry>>,
}

#[derive(Debug, Clone)]
struct Entry {
    val: Variable,
    hash: usize,
    next: Option<Box<Entry>>,
}

impl Set {
    pub fn new(args: Vec<Variable>, runtime: &mut Runtime) -> Result<Rc<Set>, ()> {
        Result::Ok(Rc::new(Set {
            value: RefCell::new(InnerSet::new(args, runtime)?),
        }))
    }

    fn from_inner(value: InnerSet) -> Rc<Set> {
        Rc::new(Set {
            value: RefCell::new(value),
        })
    }

    fn get_operator(self: &Rc<Self>, o: Operator) -> Variable {
        let func = match o {
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
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn get_attribute(self: &Rc<Self>, s: StringVar) -> Variable {
        let func = match s.as_str() {
            "add" => Self::add,
            "addAll" => Self::add_all,
            "remove" => Self::del_attr,
            "length" => return Variable::Bigint(self.value.borrow().size().into()),
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn intersection(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let other = replace(&mut args[0], Variable::Null());
        let other_iter = other.iter(runtime)?;
        let mut result_vec = Vec::new();
        while let Option::Some(val) = other_iter.next(runtime)? {
            if self.value.borrow().contains(val.clone(), runtime)? {
                result_vec.push(val);
            }
        }
        let ret = Self::new(result_vec, runtime)?;
        runtime.return_1(ret.into())
    }

    fn union(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let self_val = self.value.borrow();
        let result_vec = self_val.values.clone();
        let result_size = self_val.size;
        let mut result = InnerSet {
            values: result_vec,
            size: result_size,
        };
        let other = replace(&mut args[0], Variable::Null());
        let other_iter = other.iter(runtime)?;
        while let Option::Some(val) = other_iter.next(runtime)? {
            result.add(val, runtime)?;
        }
        runtime.return_1(Set::from_inner(result).into())
    }

    fn xor(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let self_val = self.value.borrow();
        let result_vec = self_val.values.clone();
        let result_size = self_val.size;
        let mut result = InnerSet {
            values: result_vec,
            size: result_size,
        };
        let other = replace(&mut args[0], Variable::Null());
        let other_iter = other.iter(runtime)?;
        while let Option::Some(val) = other_iter.next(runtime)? {
            if result.contains(val.clone(), runtime)? {
                result.remove(val, runtime)?;
            } else {
                result.add(val, runtime)?;
            }
        }
        runtime.return_1(Set::from_inner(result).into())
    }

    fn bool(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.is_empty().into())
    }

    fn repr(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let repr = self.value.borrow().repr(runtime)?;
        runtime.return_1(repr.into())
    }

    fn contains(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = args.remove(0);
        let is_contained = self.value.borrow().contains(val, runtime)?;
        runtime.return_1(is_contained.into())
    }

    fn add(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = args.remove(0);
        self.value.borrow_mut().add(val, runtime)?;
        runtime.return_0()
    }

    fn add_all(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = replace(&mut args[0], Variable::Null());
        let val_iter = val.iter(runtime)?;
        while let Option::Some(arg) = val_iter.next(runtime)? {
            self.value.borrow_mut().add(arg, runtime)?;
        }
        runtime.return_0()
    }

    fn del_attr(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.remove(args, runtime)?;
        runtime.pop_return();
        runtime.return_0()
    }

    fn remove(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = replace(&mut args[0], Variable::Null());
        let was_removed = self.value.borrow_mut().remove(val, runtime)?;
        runtime.return_1(was_removed.into())
    }

    fn eq(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            if !match downcast_var::<Set>(arg) {
                Option::None => false,
                Option::Some(other) => {
                    let self_val = self.value.borrow();
                    self_val.equals(&*other.value.borrow(), runtime)?
                }
            } {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn iter(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(SetIter::new(self.clone())).into())
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty()); // TODO: Set of a value
        let set = Set::new(vec![], runtime)?;
        runtime.return_1(set.into())
    }

    pub fn set_type() -> Type {
        custom_class!(Set, create, "set")
    }

    pub fn is_empty(&self) -> bool {
        self.value.borrow().is_empty()
    }

    pub fn len(&self) -> usize {
        self.value.borrow().size
    }
}

impl InnerSet {
    pub fn new(args: Vec<Variable>, runtime: &mut Runtime) -> Result<InnerSet, ()> {
        let vec_capacity = next_power_2(args.len());
        let mut value = InnerSet {
            size: 0,
            values: vec![Option::None; vec_capacity],
        };
        for x in args {
            value.add(x, runtime)?;
        }
        Result::Ok(value)
    }

    pub fn add(&mut self, arg: Variable, runtime: &mut Runtime) -> FnResult {
        let hash = arg.hash(runtime)?;
        let len = self.values.len();
        self.resize(next_power_2(self.size + 1), runtime)?;
        match &mut self.values[hash % len] {
            Option::None => {
                self.values[hash % len] = Option::Some(Entry::new(arg, hash));
                self.size += 1;
                FnResult::Ok(())
            }
            Option::Some(v) => {
                let increase_size = v.add(arg, runtime)?;
                if increase_size {
                    self.size += 1;
                }
                FnResult::Ok(())
            }
        }
    }

    pub fn remove(&mut self, arg: Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        let hash = arg.hash(runtime)?;
        let index = hash % self.values.len();
        match &mut self.values[index] {
            Option::Some(val) => {
                if val.del(&arg, runtime)? {
                    let boxed_entry = replace(&mut val.next, Option::None);
                    self.values[index] = boxed_entry.map(|x| *x);
                    Result::Ok(true)
                } else {
                    Result::Ok(false)
                }
            }
            Option::None => Result::Ok(false),
        }
    }

    pub fn repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        if self.is_empty() {
            return Result::Ok("{}".into());
        }
        let mut result = String::new();
        result += "{";
        self.for_each(|x| {
            result += x.str(runtime)?.as_str();
            result += ", ";
            FnResult::Ok(())
        })?;
        result.remove(result.len() - 1);
        result.remove(result.len() - 1);
        result += "}";
        Result::Ok(result.into())
    }

    pub fn contains(&self, val: Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        if self.is_empty() {
            return Result::Ok(false);
        }
        let hash = val.hash(runtime)?;
        let len = self.values.len();
        match &self.values[hash % len] {
            Option::None => Result::Ok(false),
            Option::Some(v) => v.contains(val, runtime),
        }
    }

    fn for_each(&self, mut func: impl FnMut(&Variable) -> FnResult) -> FnResult {
        for val in &self.values {
            if let Option::Some(o) = val {
                func(o.get_val())?;
                let mut p = o.get_next().as_ref();
                while let Option::Some(q) = p {
                    func(o.get_val())?;
                    p = q.get_next().as_ref()
                }
            }
        }
        FnResult::Ok(())
    }

    fn resize(&mut self, new_size: usize, runtime: &mut Runtime) -> FnResult {
        let current_size = self.values.len();
        if current_size >= new_size {
            return FnResult::Ok(());
        }
        let old_vec = replace(&mut self.values, vec![Option::None; new_size]);
        for entry in old_vec {
            if let Option::Some(mut e) = entry {
                loop {
                    let (entry, next) = Self::split_entries(e);
                    self.add(entry.val, runtime)?;
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

    pub fn equals(&self, other: &InnerSet, runtime: &mut Runtime) -> Result<bool, ()> {
        if self.size != other.size {
            return Result::Ok(false);
        }
        for val in &self.values {
            if let Option::Some(o) = val {
                if !other.contains(o.val.clone(), runtime)? {
                    return Result::Ok(false);
                }
                let mut p = o.get_next().as_ref();
                while let Option::Some(q) = p {
                    if !other.contains(q.val.clone(), runtime)? {
                        return Result::Ok(false);
                    }
                    p = q.get_next().as_ref()
                }
            }
        }
        Result::Ok(true)
    }

    fn split_entries(mut e: Entry) -> (Entry, Option<Box<Entry>>) {
        let next = replace(&mut e.next, Option::None);
        (e, next)
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Entry {
    pub fn new(val: Variable, hash: usize) -> Entry {
        Entry {
            val,
            hash,
            next: Option::None,
        }
    }

    pub fn add(&mut self, val: Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        if self.val.equals(val.clone(), runtime)? {
            self.val = val;
            Result::Ok(false)
        } else {
            match &mut self.next {
                Option::None => {
                    let hash = val.hash(runtime)?;
                    self.next = Option::Some(Box::new(Entry::new(val, hash)));
                    Result::Ok(true)
                }
                Option::Some(v) => v.add(val, runtime),
            }
        }
    }

    pub fn del(&mut self, key: &Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        if key.equals(self.val.clone(), runtime)? {
            Result::Ok(true)
        } else {
            match &mut self.next {
                Option::None => Result::Ok(false),
                Option::Some(e) => {
                    if e.del(key, runtime)? {
                        self.next = replace(&mut e.next, Option::None);
                        Result::Ok(true)
                    } else {
                        Result::Ok(false)
                    }
                }
            }
        }
    }

    pub fn get_val(&self) -> &Variable {
        &self.val
    }

    pub fn get_next(&self) -> &Option<Box<Entry>> {
        &self.next
    }

    pub fn contains(&self, val: Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        if self.val.equals(val.clone(), runtime)? {
            Result::Ok(true)
        } else {
            match &self.next {
                Option::None => Result::Ok(false),
                Option::Some(v) => v.contains(val, runtime),
            }
        }
    }

    fn get_entry(&self, key: Variable, runtime: &mut Runtime) -> Result<&Entry, ()> {
        if self.val.equals(key.clone(), runtime)? {
            Result::Ok(self)
        } else {
            self.next.as_ref().unwrap().get_entry(key, runtime)
        }
    }
}

impl CustomVar for Set {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Attribute(s) => self.get_attribute(s),
            Name::Operator(o) => self.get_operator(o),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Set::set_type()
    }
}

#[derive(Debug)]
struct SetIter {
    parent: Rc<Set>,
    bucket_no: Cell<usize>,
    index: RefCell<Variable>,
}

impl SetIter {
    fn new(parent: Rc<Set>) -> SetIter {
        let val = SetIter {
            parent,
            bucket_no: Cell::new(0),
            index: RefCell::new(Variable::Null()),
        };
        match val.parent.value.borrow().values[0].as_ref() {
            Option::Some(entry) => {
                val.index.replace(entry.get_val().clone());
            }
            Option::None => val.point_to_next(),
        }
        val
    }

    fn point_to_next(&self) {
        let parent = self.parent.value.borrow();
        self.bucket_no.set(self.bucket_no.get() + 1);
        while self.bucket_no.get() < parent.size {
            if let Option::Some(val) = parent.values[self.bucket_no.get()].as_ref() {
                self.index.replace(val.get_val().clone());
                return;
            }
            self.bucket_no.set(self.bucket_no.get() + 1);
        }
    }

    fn next_fn(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let next = self.clone().next(runtime)?;
        runtime.return_1(next.into())
    }
}

impl CustomVar for SetIter {
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

impl NativeIterator for SetIter {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        let len = self.parent.len();
        let bucket = self.bucket_no.get();
        if bucket >= len {
            return Result::Ok(Option::None);
        }
        let parent = self.parent.value.borrow();
        let parent_node = parent.values[bucket].as_ref().unwrap();
        let node = parent_node.get_entry(self.index.borrow().clone(), runtime)?;
        let val = self.index.replace(Variable::Null());
        debug_assert!(node.get_val().equals(val.clone(), runtime)?);
        if let Option::Some(next) = node.get_next() {
            self.index.replace(next.get_val().clone());
        } else {
            self.point_to_next();
        }
        Result::Ok(Option::Some(val))
    }
}
