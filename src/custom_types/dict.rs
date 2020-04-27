use crate::custom_var::{CustomVar, CustomVarWrapper};
use crate::method::StdMethod;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use std::cell::RefCell;
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

fn vec_size(len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let leading0s: u32 = len.leading_zeros();
    const TOTAL_ZEROS: u32 = usize::leading_zeros(0);
    (1 as usize) << (TOTAL_ZEROS - leading0s) as usize
}

impl Dict {
    pub fn from_args(keys: Vec<Variable>, values: Vec<Variable>, runtime: &mut Runtime) -> Dict {
        Dict {
            value: Rc::new(RefCell::new(InnerDict::from_args(keys, values, runtime))),
        }
    }

    fn get_op(&self, o: Operator) -> Variable {
        let func = match o {
            Operator::GetAttr => Dict::index,
            Operator::Repr => Dict::repr,
            Operator::Str => Dict::repr,
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
        let repr = self.true_repr(runtime)?;
        runtime.push(repr.into());
        FnResult::Ok(())
    }

    fn true_repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        if self.is_empty() {
            return Result::Ok("{:}".into());
        }
        let mut result = String::new();
        result += "{";
        for val in (*self.value).borrow().get_entries() {
            if let Option::Some(o) = val {
                result += o.get_key().str(runtime)?.as_str();
                result += ": ";
                result += o.get_value().str(runtime)?.as_str();
                result += ", ";
                let mut p = o.get_next().as_ref();
                while let Option::Some(q) = p {
                    result += q.get_key().str(runtime)?.as_str();
                    result += ": ";
                    result += q.get_value().str(runtime)?.as_str();
                    result += ", ";
                    p = q.get_next().as_ref()
                }
            }
        }
        result.remove(result.len() - 1);
        result.remove(result.len() - 1);
        result += "}";
        Result::Ok(result.into())
    }

    fn is_empty(&self) -> bool {
        (*self.value).borrow().is_empty()
    }
}

impl InnerDict {
    pub fn from_args(
        keys: Vec<Variable>,
        values: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> InnerDict {
        debug_assert!(keys.len() == values.len());
        let vec_capacity = vec_size(keys.len());
        let mut value = InnerDict {
            size: keys.len(),
            entries: Vec::with_capacity(vec_capacity as usize),
        };
        for (x, y) in keys.into_iter().zip(values) {
            let entry = Entry::new(x, y, 0);
            value.add_var(entry, runtime);
        }
        value
    }

    fn add_var(&mut self, value: Entry, runtime: &mut Runtime) -> bool {
        let index = value.get_hash() % self.entries.len();
        return match &mut self.entries[index] {
            Option::None => {
                self.entries[index] = Option::Some(value);
                true
            }
            Option::Some(v) => v.set_entry(value, runtime),
        };
    }

    pub fn get(&self, key: Variable, runtime: &mut Runtime) -> Result<Variable, ()> {
        let hash = key.hash(runtime)?;
        return match &self.entries[hash % self.entries.len()] {
            Option::None => Result::Err(()),
            Option::Some(e) => e.get(key, runtime).ok_or(()),
        };
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub(self) fn get_entries(&self) -> &Vec<Option<Entry>> {
        &self.entries
    }
}

impl Entry {
    pub fn new(key: Variable, value: Variable, hash: usize) -> Entry {
        Entry {
            key,
            value,
            hash,
            next: Option::None,
        }
    }

    pub fn get_hash(&self) -> &usize {
        &self.hash
    }

    pub fn set_entry(&mut self, entry: Entry, runtime: &mut Runtime) -> bool {
        let replace_this = self.value.equals(entry.value.clone(), runtime);
        if replace_this {
            debug_assert_eq!(self.hash, entry.hash);
            self.value = entry.value;
            false
        } else {
            return match &mut self.next {
                Option::None => {
                    self.next = Option::Some(Box::new(entry));
                    true
                }
                Option::Some(e) => e.set_entry(entry, runtime),
            };
        }
    }

    pub fn get(&self, key: Variable, runtime: &mut Runtime) -> Option<Variable> {
        if key.equals(self.value.clone(), runtime) {
            Option::Some(self.value.clone())
        } else {
            return match &self.next {
                Option::None => Option::None,
                Option::Some(e) => e.get(key, runtime),
            };
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
            Name::Attribute(_) => unimplemented!(),
        }
    }

    fn set(&mut self, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        unimplemented!()
    }
}

impl From<Dict> for Variable {
    fn from(val: Dict) -> Self {
        Variable::Custom(CustomVarWrapper::new(Box::new(val)))
    }
}
