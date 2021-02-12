use crate::custom_var::{downcast_var, CustomVar};
use crate::first;
use crate::int_tools::next_power_2;
use crate::int_var::IntVar;
use crate::looping::{self, IterResult, NativeIterator};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::cell::{Cell, RefCell};
use std::mem::{replace, take};
use std::rc::Rc;

#[derive(Debug)]
pub struct Set {
    generic: Type,
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
    pub fn new(generic: Type, args: Vec<Variable>, runtime: &mut Runtime) -> Result<Rc<Set>, ()> {
        Result::Ok(Rc::new(Set {
            generic,
            value: RefCell::new(InnerSet::new(args, runtime)?),
        }))
    }

    fn from_inner(generic: Type, value: InnerSet) -> Rc<Set> {
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

    fn get_operator(self: Rc<Self>, o: Operator) -> Variable {
        let func = Set::op_fn(o);
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, s: &str) -> Variable {
        let func = match s {
            "add" => Self::add,
            "addAll" => Self::add_all,
            "remove" => Self::del_attr,
            "clear" => Self::clear,
            "length" => return IntVar::from(self.value.borrow().size()).into(),
            _ => unimplemented!(),
        };
        StdMethod::new_native(self, func).into()
    }

    fn intersection(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let other = first(args);
        let other_iter = other.iter(runtime)?;
        let mut result_vec = Vec::new();
        while let Option::Some(val) = other_iter.next(runtime)?.take_first() {
            if self.value.borrow().contains(val.clone(), runtime)? {
                result_vec.push(val);
            }
        }
        let ret = Self::new(self.generic, result_vec, runtime)?;
        runtime.return_1(ret.into())
    }

    fn union(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let self_val = self.value.borrow();
        let result_vec = self_val.values.clone();
        let result_size = self_val.size;
        let mut result = InnerSet {
            values: result_vec,
            size: result_size,
        };
        let other = first(args);
        let other_iter = other.iter(runtime)?;
        while let Option::Some(val) = other_iter.next(runtime)?.take_first() {
            result.add(val, runtime)?;
        }
        runtime.return_1(Set::from_inner(self.generic, result).into())
    }

    fn xor(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let self_val = self.value.borrow();
        let result_vec = self_val.values.clone();
        let result_size = self_val.size;
        let mut result = InnerSet {
            values: result_vec,
            size: result_size,
        };
        let other = first(args);
        let other_iter = other.iter(runtime)?;
        while let Option::Some(val) = other_iter.next(runtime)?.take_first() {
            if result.contains(val.clone(), runtime)? {
                result.remove(val, runtime)?;
            } else {
                result.add(val, runtime)?;
            }
        }
        runtime.return_1(Set::from_inner(self.generic, result).into())
    }

    fn bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.is_empty().into())
    }

    fn repr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let repr = self.value.borrow().repr(runtime)?;
        runtime.return_1(repr.into())
    }

    fn contains(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = first(args);
        let is_contained = self.value.borrow().contains(val, runtime)?;
        runtime.return_1(is_contained.into())
    }

    fn add(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = first(args);
        if val.get_type().is_subclass(&self.generic, runtime) {
            self.value.borrow_mut().add(val, runtime)?;
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
                self.value.borrow_mut().add(arg, runtime)?;
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
        self.remove(args, runtime)?;
        runtime.pop_return();
        runtime.return_0()
    }

    fn remove(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = first(args);
        let was_removed = self.value.borrow_mut().remove(val, runtime)?;
        runtime.return_1(was_removed.into())
    }

    fn eq(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
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

    fn iter(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(SetIter::new(self)).into())
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty()); // TODO: Set of a value
        let set = Set::new(Type::Object, vec![], runtime)?;
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
        let hash = arg.clone().hash(runtime)?;
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
        let hash = arg.clone().hash(runtime)?;
        let index = hash % self.values.len();
        match &mut self.values[index] {
            Option::Some(val) => {
                if val.del(&arg, runtime)? {
                    let boxed_entry = val.next.take();
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
        for x in self {
            result += x.clone().str(runtime)?.as_str();
            result += ", ";
        }
        result.remove(result.len() - 1);
        result.remove(result.len() - 1);
        result += "}";
        Result::Ok(result.into())
    }

    pub fn contains(&self, val: Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        if self.is_empty() {
            return Result::Ok(false);
        }
        let hash = val.clone().hash(runtime)?;
        let len = self.values.len();
        match &self.values[hash % len] {
            Option::None => Result::Ok(false),
            Option::Some(v) => v.contains(val, runtime),
        }
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
                    let next = e.next.take();
                    self.add(e.val, runtime)?;
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
        for val in self {
            if !other.contains(val.clone(), runtime)? {
                return Result::Ok(false);
            }
        }
        Result::Ok(true)
    }

    pub fn clear(&mut self) {
        self.size = 0;
        for val in &mut self.values {
            val.take();
        }
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
        if self.val.clone().equals(val.clone(), runtime)? {
            self.val = val;
            Result::Ok(false)
        } else {
            match &mut self.next {
                Option::None => {
                    let hash = val.clone().hash(runtime)?;
                    self.next = Option::Some(Box::new(Entry::new(val, hash)));
                    Result::Ok(true)
                }
                Option::Some(v) => v.add(val, runtime),
            }
        }
    }

    pub fn del(&mut self, key: &Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        if key.clone().equals(self.val.clone(), runtime)? {
            Result::Ok(true)
        } else {
            match &mut self.next {
                Option::None => Result::Ok(false),
                Option::Some(e) => {
                    if e.del(key, runtime)? {
                        self.next = e.next.take();
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
        if self.val.clone().equals(val.clone(), runtime)? {
            Result::Ok(true)
        } else {
            match &self.next {
                Option::None => Result::Ok(false),
                Option::Some(v) => v.contains(val, runtime),
            }
        }
    }

    fn get_entry(&self, key: Variable, runtime: &mut Runtime) -> Result<&Entry, ()> {
        if self.val.clone().equals(key.clone(), runtime)? {
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

    fn get_type(&self) -> Type {
        Set::set_type()
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
        self.value.borrow().repr(runtime)
    }

    fn repr(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.value.borrow().repr(runtime)
    }

    fn bool(self: Rc<Self>, _runtime: &mut Runtime) -> Result<bool, ()> {
        Result::Ok(!self.value.borrow().is_empty())
    }

    fn iter(self: Rc<Self>, _runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        Result::Ok(Rc::new(SetIter::new(self)).into())
    }
}

impl<'a> IntoIterator for &'a InnerSet {
    type Item = &'a Variable;
    type IntoIter = InnerSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        for (i, value) in self.values.iter().enumerate() {
            if let Option::Some(x) = value {
                return InnerSetIter {
                    parent: self,
                    i,
                    current: Option::Some(x),
                };
            }
        }
        InnerSetIter {
            parent: self,
            i: self.values.len(),
            current: Option::None,
        }
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
            index: RefCell::new(Variable::null()),
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

    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let next = self.next(runtime)?;
        runtime.return_1(next.take_first().into())
    }
}

impl CustomVar for SetIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        let func = match name {
            Name::Operator(_) => unimplemented!(),
            Name::Attribute(val) => match val {
                "next" => Self::next_fn,
                _ => unimplemented!(),
            },
        };
        StdMethod::new_native(self, func).into()
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        unimplemented!()
    }
}

impl NativeIterator for SetIter {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        let len = self.parent.len();
        let bucket = self.bucket_no.get();
        if bucket >= len {
            return Result::Ok(Option::<Variable>::None.into());
        }
        let parent = self.parent.value.borrow();
        let parent_node = parent.values[bucket].as_ref().unwrap();
        let node = parent_node.get_entry(self.index.borrow().clone(), runtime)?;
        let val = self.index.replace(Variable::default());
        debug_assert!(node.get_val().clone().equals(val.clone(), runtime)?);
        if let Option::Some(next) = node.get_next() {
            self.index.replace(next.get_val().clone());
        } else {
            self.point_to_next();
        }
        Result::Ok(Option::Some(val).into())
    }
}

struct InnerSetIter<'a> {
    parent: &'a InnerSet,
    i: usize,
    current: Option<&'a Entry>,
}

impl InnerSetIter<'_> {
    fn adjust_i(&mut self) {
        self.i += 1;
        while self.i < self.parent.values.len() && self.parent.values[self.i].is_none() {
            self.i += 1;
        }
    }
}

impl<'a> Iterator for InnerSetIter<'a> {
    type Item = &'a Variable;

    fn next(&mut self) -> Option<Self::Item> {
        match take(&mut self.current) {
            Option::None => Option::None,
            Option::Some(entry) => {
                self.current = match &entry.next {
                    Option::Some(x) => Option::Some(&**x),
                    Option::None => {
                        self.adjust_i();
                        self.parent.values[self.i].as_ref()
                    }
                };
                Option::Some(&entry.val)
            }
        }
    }
}
