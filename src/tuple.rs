use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::Variable;
use downcast_rs::__std::ops::Index;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LangTuple {
    values: Rc<Vec<Variable>>,
}

impl LangTuple {
    pub fn new(args: Vec<Variable>) -> Self {
        LangTuple {
            values: Rc::new(args),
        }
    }

    pub fn str(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let mut result = "(".to_string();
        for (i, value) in self.values.iter().enumerate() {
            result += value.str(runtime)?.as_str();
            if i < self.values.len() - 1 {
                result += ", ".into();
            }
        }
        Result::Ok(StringVar::from(result))
    }

    pub fn repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let mut result = "(".to_string();
        for (i, value) in self.values.iter().enumerate() {
            result += value.repr(runtime)?.as_str();
            if i < self.values.len() - 1 {
                result += ", ".into();
            }
        }
        Result::Ok(StringVar::from(result))
    }

    pub fn is_empty(&self) -> bool {
        self.values.len() == 0
    }

    pub fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.values, &other.values)
    }

    pub fn lang_hash(&self, runtime: &mut Runtime) -> Result<usize, ()> {
        // Copied from Python's tuple hash algorithm
        let mut x: usize = 0x345678;
        let mul = 1000003;
        for value in &*self.values {
            let y = value.hash(runtime)?;
            x = (x ^ y).wrapping_mul(mul);
        }
        Result::Ok(x)
    }

    pub fn id(&self) -> usize {
        Rc::as_ptr(&self.values) as *const () as usize
    }

    pub fn get_values(&self) -> &Vec<Variable> {
        &*self.values
    }
}

impl Index<usize> for LangTuple {
    type Output = Variable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl Hash for LangTuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in &*self.values {
            Hash::hash(value, state);
        }
    }
}
