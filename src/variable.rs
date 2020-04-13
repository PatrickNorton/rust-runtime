use std::rc::Rc;
use std::vec::Vec;
use std::string::String;
use std::cell::RefCell;
use std::boxed::Box;
use std::clone::Clone;
use std::cmp::PartialEq;

use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use num_bigint::{BigInt, ToBigInt};
use bigdecimal::BigDecimal;
use crate::method::Method;

pub enum Name {
    Attribute(String),
    Operator(Operator),
}

#[derive(Clone, Eq, Hash)]
pub enum Variable {
    Bigint(BigInt),
    String(String),
    Decimal(BigDecimal),
    Type(Type),
    Standard(StdVariable),
    Method(Box<dyn Method>),
    Custom(),
}

impl Variable {
    pub fn str(&mut self, runtime: &mut Runtime) -> String {
        return match self {
            Variable::String(val) => val.clone(),
            Variable::Bigint(val) => val.to_str_radix(10),
            Variable::Decimal(val) => val.to_string(),
            Variable::Type(val) => val.to_string(),
            Variable::Standard(val) => val.clone().str(runtime),
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

    pub fn call(&self, args: (&Vec<Variable>, &mut Runtime)) {
        match self {
            Variable::Standard(val) => val.call(args),
            Variable::Method(method) => method.call(args),
            _ => unimplemented!()
        }
    }

    pub fn index(&self, index: Name) -> Variable {
        return match self {
            Variable::Standard(val) => val.index(index),
            _ => unimplemented!()
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        return match self {
            Variable::Bigint(val) =>
                if let Variable::Bigint(o) = other {
                    val == o
                } else { false }
            Variable::String(val) =>
                if let Variable::String(o) = other {
                    val == o
                } else { false }
            Variable::Decimal(val) =>
                if let Variable::Decimal(o) = other {
                    val == o
                } else { false }
            Variable::Type(val) =>
                if let Variable::Type(o) = other {
                    val == o
                } else { false }
            Variable::Standard(val) =>
                if let Variable::Standard(o) = other {
                    val == o
                } else { false }
            Variable::Method(val) =>
                if let Variable::Method(o) = other {
                    val == o
                } else { false }
            Variable::Custom() => unimplemented!()
        }
    }
}
