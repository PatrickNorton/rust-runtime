use crate::custom_types::types::CustomType;
use crate::custom_var::{downcast_var, CustomVar};
use crate::function::Function;
use crate::int_tools::next_power_2;
use crate::method::StdMethod;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Iterator;
use std::mem::replace;
use std::ops::Deref;
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
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn get_attribute(self: &Rc<Self>, s: StringVar) -> Variable {
        let func = match s.as_str() {
            "clear" => Dict::clear,
            "get" => Dict::get,
            "length" => return Variable::Bigint(self.len().into()),
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn index(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        match self.value.borrow().get(args.remove(0), runtime)? {
            Option::Some(result) => {
                runtime.push(result);
                FnResult::Ok(())
            }
            Option::None => runtime.throw_quick(Type::String, "Value not found".into()),
        }
    }

    fn repr(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let repr = self.value.borrow().true_repr(runtime)?;
        runtime.push(repr.into());
        FnResult::Ok(())
    }

    fn bool(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push((!self.is_empty()).into());
        FnResult::Ok(())
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
        runtime.push(is_in.into());
        FnResult::Ok(())
    }

    fn clear(self: &Rc<Self>, args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().clear();
        FnResult::Ok(())
    }

    fn get(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let val = match self.value.borrow().get(args[0].clone(), runtime)? {
            Option::Some(value) => value,
            Option::None => args[1].clone(),
        };
        runtime.push(val);
        FnResult::Ok(())
    }

    fn eq(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            match downcast_var::<Dict>(arg) {
                Option::None => {
                    runtime.push(false.into());
                    return FnResult::Ok(());
                }
                Option::Some(other) => {
                    let self_val = self.value.borrow();
                    if !self_val.equals(&*other.value.borrow(), runtime)? {
                        runtime.push(false.into());
                        return FnResult::Ok(());
                    }
                }
            };
        }
        runtime.push(true.into());
        FnResult::Ok(())
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty()); // TODO: List of a value
        let dict = Dict::from_args(Vec::new(), Vec::new(), runtime)?;
        runtime.push(dict.into());
        FnResult::Ok(())
    }

    pub fn dict_type() -> Type {
        lazy_static! {
            static ref TYPE: CustomType<Dict> = CustomType::new(
                "list".into(),
                Vec::new(),
                Function::Native(Dict::create),
                HashMap::new()
            );
        }
        Type::Custom(&*TYPE)
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
            result += x.str(runtime)?.as_str();
            result += ": ";
            result += y.str(runtime)?.as_str();
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
                Result::Ok(())
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

    pub fn get_key(&self) -> &Variable {
        &self.key
    }

    pub fn get_value(&self) -> &Variable {
        &self.value
    }

    pub fn get_next(&self) -> &Option<Box<Entry>> {
        &self.next
    }
}

impl CustomVar for Dict {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(o) => self.get_op(o),
            Name::Attribute(s) => self.get_attribute(s),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Dict::dict_type()
    }
}
