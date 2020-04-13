use std::rc::Rc;
use std::vec::Vec;
use std::string::String;
use std::cell::RefCell;

use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use num_bigint::{BigInt, ToBigInt};
use bigdecimal::BigDecimal;

pub enum Method<T> {
    Standard(i32),
    Native(fn(T, Vec<Variable>, &Runtime)),
}

pub enum Variable {
    Bigint(BigInt),
    String(String),
    Decimal(BigDecimal),
    Type(Type),
    Standard(Rc<RefCell<StdVariable>>),
    Custom(),
}

impl Variable {
    pub fn str(&mut self, runtime: &Runtime) -> String {
        return match self {
            Variable::String(val) => val.clone(),
            Variable::Bigint(val) => val.to_str_radix(10),
            Variable::Decimal(val) => val.to_string(),
            Variable::Type(val) => val.to_string(),
            Variable::Standard(val) => val.borrow_mut().str(runtime),
            _ => unimplemented!()
        }
    }

    pub fn int(&mut self, _runtime: &Runtime) -> BigInt {
        return match self {
            Variable::Bigint(val) => val.clone(),
            Variable::Decimal(val) => val.to_bigint().unwrap(),
            _ => unimplemented!()
        }
    }
}
