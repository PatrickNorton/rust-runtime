use std::boxed::Box;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::string::String;
use std::vec::Vec;

use crate::bytecode::Bytecode::TailTos;
use crate::method::Method;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use num::bigint::{BigInt, ToBigInt};
use num::{BigRational, Rational};
use num_traits::Zero;

pub enum Name {
    Attribute(String),
    Operator(Operator),
}

#[derive(Clone, Eq, Hash)]
pub enum Variable {
    Null(),
    Bool(bool),
    Bigint(BigInt),
    String(String),
    Decimal(BigRational),
    Type(Type),
    Standard(StdVariable),
    Method(Box<dyn Method>),
    Custom(),
}

impl Variable {
    pub fn str(&self, runtime: &mut Runtime) -> String {
        return match self {
            Variable::Null() => String::from("null"),
            Variable::Bool(val) => String::from(if *val { "true" } else { "false" }),
            Variable::String(val) => val.clone(),
            Variable::Bigint(val) => val.to_str_radix(10),
            Variable::Decimal(val) => val.to_string(),
            Variable::Type(val) => val.to_string(),
            Variable::Standard(val) => val.clone().str(runtime),
            _ => unimplemented!(),
        };
    }

    pub fn int(&self, _runtime: &Runtime) -> BigInt {
        return match self {
            Variable::Bigint(val) => val.clone(),
            Variable::Decimal(val) => val.to_integer(),
            _ => unimplemented!(),
        };
    }

    pub fn to_bool(&self, _runtime: &mut Runtime) -> bool {
        return match self {
            Variable::Null() => false,
            Variable::Bool(val) => *val,
            Variable::String(val) => !val.is_empty(),
            Variable::Bigint(val) => val == &BigInt::zero(),
            Variable::Decimal(val) => val == &BigRational::zero(),
            Variable::Type(val) => true,
            Variable::Standard(val) => val.clone().bool(_runtime),
            Variable::Method(_) => true,
            Variable::Custom() => unimplemented!(),
        };
    }

    pub fn call(&self, args: (&Vec<Variable>, &mut Runtime)) {
        match self {
            Variable::Standard(val) => val.call(args),
            Variable::Method(method) => method.call(args),
            _ => unimplemented!(),
        }
    }

    pub fn index(&self, index: Name) -> Variable {
        return match self {
            Variable::Standard(val) => val.index(index),
            _ => unimplemented!(),
        };
    }

    pub fn set(&self, index: String, value: Variable, _runtime: &mut Runtime) {
        match self {
            Variable::Standard(val) => val.set(index, value),
            Variable::Custom() => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            Variable::Null() => Type::Null(),
            Variable::Bool(_) => Type::Bool(),
            Variable::String(_) => Type::String(),
            Variable::Bigint(_) => Type::Bigint(),
            Variable::Decimal(_) => Type::Decimal(),
            Variable::Type(_) => Type::Type(),
            Variable::Method(_) => unimplemented!(),
            Variable::Standard(a) => a.get_type(),
            Variable::Custom() => unimplemented!(),
        }
    }

    pub fn identical(&self, other: &Variable) -> bool {
        return match (self, other) {
            (Variable::Null(), Variable::Null()) => true,
            (Variable::Bool(a), Variable::Bool(b)) => a == b,
            (Variable::String(a), Variable::String(b)) => a == b,
            (Variable::Bigint(a), Variable::Bigint(b)) => a == b,
            (Variable::Decimal(a), Variable::Decimal(b)) => a == b,
            (Variable::Type(a), Variable::Type(b)) => a == b,
            (Variable::Standard(a), Variable::Standard(b)) => a.identical(b),
            (Variable::Method(a), Variable::Method(b)) => a == b,
            (Variable::Custom(), Variable::Custom()) => unimplemented!(),
            _ => false,
        };
    }

    pub fn is_type_of(&self, other: &Variable) -> bool {
        if let Variable::Type(t) = self {
            t.is_type_of(other)
        } else {
            false
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        return match self {
            Variable::Null() => {
                if let Variable::Null() = other {
                    true
                } else {
                    false
                }
            }
            Variable::Bool(val) => {
                if let Variable::Bool(o) = other {
                    val == o
                } else {
                    false
                }
            }
            Variable::Bigint(val) => {
                if let Variable::Bigint(o) = other {
                    val == o
                } else {
                    false
                }
            }
            Variable::String(val) => {
                if let Variable::String(o) = other {
                    val == o
                } else {
                    false
                }
            }
            Variable::Decimal(val) => {
                if let Variable::Decimal(o) = other {
                    val == o
                } else {
                    false
                }
            }
            Variable::Type(val) => {
                if let Variable::Type(o) = other {
                    val == o
                } else {
                    false
                }
            }
            Variable::Standard(val) => {
                if let Variable::Standard(o) = other {
                    val == o
                } else {
                    false
                }
            }
            Variable::Method(val) => {
                if let Variable::Method(o) = other {
                    val == o
                } else {
                    false
                }
            }
            Variable::Custom() => unimplemented!(),
        };
    }
}
