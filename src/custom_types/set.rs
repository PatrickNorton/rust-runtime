use crate::custom_types::types::CustomType;
use crate::custom_var::CustomVar;
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

    fn get_operator(self: &Rc<Self>, o: Operator) -> Variable {
        let func = match o {
            Operator::Bool => Self::bool,
            Operator::Str => Self::repr,
            Operator::Repr => Self::repr,
            Operator::In => Self::contains,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn get_attribute(self: &Rc<Self>, s: StringVar) -> Variable {
        let func = match s.as_str() {
            "add" => Self::add,
            "length" => return Variable::Bigint(self.value.borrow().size().into()),
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn bool(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push(self.is_empty().into());
        FnResult::Ok(())
    }

    fn repr(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let repr = self.value.borrow().repr(runtime)?;
        runtime.push(repr.into());
        FnResult::Ok(())
    }

    fn contains(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = args.remove(0);
        let is_contained = self.value.borrow().contains(val, runtime)?;
        runtime.push(is_contained.into());
        FnResult::Ok(())
    }

    fn add(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = args.remove(0);
        self.value.borrow_mut().add(val, runtime)?;
        FnResult::Ok(())
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty()); // TODO: Set of a value
        let set = Set::new(vec![], runtime)?;
        runtime.push(set.into());
        FnResult::Ok(())
    }

    pub fn set_type() -> Type {
        lazy_static! {
            static ref TYPE: CustomType<Set> = CustomType::new(
                "list".into(),
                Vec::new(),
                Function::Native(Set::create),
                HashMap::new()
            );
        }
        Type::Custom(&*TYPE)
    }

    pub fn is_empty(&self) -> bool {
        self.value.borrow().is_empty()
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
