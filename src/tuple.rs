use crate::custom_types::join_values;
use crate::runtime::Runtime;
use crate::string_var::{MaybeString, StringVar};
use crate::variable::Variable;
use ascii::AsciiStr;
use once_cell::sync::Lazy;
use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Index;
use std::rc::Rc;
use std::slice;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LangTuple {
    values: Rc<[Variable]>,
}

impl LangTuple {
    pub fn new(args: Rc<[Variable]>) -> Self {
        LangTuple { values: args }
    }

    pub fn from_vec(args: Vec<Variable>) -> Self {
        LangTuple::new(args.into_boxed_slice().into())
    }

    pub fn str(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        static OPEN_PAREN: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii("(").unwrap());
        static CLOSE_PAREN: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii(")").unwrap());
        let mut result = MaybeString::Ascii(OPEN_PAREN.to_owned());
        result += &join_values(&self.values, |x| x.str(runtime))?;
        result += *CLOSE_PAREN;
        Result::Ok(StringVar::from(result))
    }

    pub fn repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let mut result = "(".to_string();
        for (i, value) in self.values.iter().enumerate() {
            result += value.clone().repr(runtime)?.as_str();
            if i < self.values.len() - 1 {
                result += ", ";
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
            let y = value.clone().hash(runtime)?;
            x = (x ^ y).wrapping_mul(mul);
        }
        Result::Ok(x)
    }

    pub fn id(&self) -> usize {
        Rc::as_ptr(&self.values) as *const () as usize
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn iter(&self) -> slice::Iter<Variable> {
        self.values.iter()
    }
}

impl Borrow<[Variable]> for LangTuple {
    fn borrow(&self) -> &[Variable] {
        self.values.borrow()
    }
}

impl<'a> IntoIterator for &'a LangTuple {
    type Item = &'a Variable;
    type IntoIter = slice::Iter<'a, Variable>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Index<usize> for LangTuple {
    type Output = Variable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}
