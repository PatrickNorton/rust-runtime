use crate::custom_types::types::CustomType;
use crate::custom_var::{CustomVar, CustomVarWrapper};
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
use std::rc::Rc;

#[derive(Debug)]
struct Entry {
    key: Variable,
    value: Variable,
    hash: usize,
    next: Option<Box<Entry>>,
}

#[derive(Debug, Clone)]
pub struct Dict {
    value: Rc<RefCell<InnerDict>>,
}

#[derive(Debug)]
pub struct InnerDict {
    size: usize,
    entries: Vec<Option<Entry>>,
}

impl Dict {
    pub fn from_args(
        keys: Vec<Variable>,
        values: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> Result<Dict, ()> {
        Result::Ok(Dict {
            value: Rc::new(RefCell::new(InnerDict::from_args(keys, values, runtime)?)),
        })
    }

    fn get_op(&self, o: Operator) -> Variable {
        let func = match o {
            Operator::GetAttr => Dict::index,
            Operator::Repr => Dict::repr,
            Operator::Str => Dict::repr,
            Operator::Bool => Dict::bool,
            Operator::SetAttr => Dict::set,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn get_attr(&self, s: StringVar) -> Variable {
        let func = match s.as_str() {
            "clear" => Dict::clear,
            "length" => return Variable::Bigint(self.len().into()),
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn index(&self, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let result = (*self.value).borrow().get(args.remove(0), runtime)?;
        runtime.push(result);
        FnResult::Ok(())
    }

    fn repr(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let repr = (*self.value).borrow().true_repr(runtime)?;
        runtime.push(repr.into());
        FnResult::Ok(())
    }

    fn bool(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push((!self.is_empty()).into());
        FnResult::Ok(())
    }

    fn set(&self, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let val = args.remove(1); // Reverse order to avoid move
        let key = args.remove(0);
        (*self.value).borrow_mut().set(key, val, runtime)
    }

    fn clear(&self, args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().clear();
        FnResult::Ok(())
    }

    pub fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
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
        (*self.value).borrow().is_empty()
    }

    fn len(&self) -> usize {
        (*self.value).borrow().size
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
            entries: Vec::with_capacity(vec_capacity as usize),
        };
        for (x, y) in keys.into_iter().zip(values) {
            value.set(x, y, runtime)?;
        }
        Result::Ok(value)
    }

    pub fn get(&self, key: Variable, runtime: &mut Runtime) -> Result<Variable, ()> {
        let hash = key.hash(runtime)?;
        match &self.entries[hash % self.entries.len()] {
            Option::None => Result::Err(()),
            Option::Some(e) => e.get(key, runtime).ok_or(()),
        }
    }

    fn true_repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        if self.is_empty() {
            return Result::Ok("{:}".into());
        }
        let mut result = String::new();
        result += "{";
        self.for_each(&mut |x, y| {
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

    fn for_each(&self, func: &mut dyn FnMut(&Variable, &Variable) -> FnResult) -> FnResult {
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

    pub fn clear(&mut self) {
        self.size = 0;
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl Entry {
    pub fn get(&self, key: Variable, runtime: &mut Runtime) -> Option<Variable> {
        if key.equals(self.value.clone(), runtime) {
            Option::Some(self.value.clone())
        } else {
            match &self.next {
                Option::None => Option::None,
                Option::Some(e) => e.get(key, runtime),
            }
        }
    }

    pub fn set(&mut self, key: Variable, val: Variable, runtime: &mut Runtime) -> Option<bool> {
        if key.equals(self.value.clone(), runtime) {
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
    fn get_attr(&self, name: Name) -> Variable {
        match name {
            Name::Operator(o) => self.get_op(o),
            Name::Attribute(s) => self.get_attr(s),
        }
    }

    fn set(&self, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Dict::dict_type()
    }
}

impl From<Dict> for Variable {
    fn from(val: Dict) -> Self {
        Variable::Custom(CustomVarWrapper::new(Box::new(val)))
    }
}
