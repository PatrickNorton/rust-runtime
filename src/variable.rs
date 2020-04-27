use std::boxed::Box;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::string::String;
use std::vec::Vec;

use crate::custom_var::CustomVarWrapper;
use crate::file_info::FileInfo;
use crate::int_functions::get_operator;
use crate::method::Method;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use crate::string_functions;
use crate::string_var::StringVar;
use num::bigint::BigInt;
use num::traits::Zero;
use num::BigRational;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::str::FromStr;

pub type FnResult = Result<(), ()>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Name {
    Attribute(StringVar),
    Operator(Operator),
}

#[derive(Clone)]
pub enum Function {
    Standard(usize, u32),
    Native(fn(Vec<Variable>, &mut Runtime) -> FnResult),
}

#[derive(Debug, Clone, Hash)]
pub enum Variable {
    Null(),
    Bool(bool),
    Bigint(BigInt),
    String(StringVar),
    Decimal(BigRational),
    Type(Type),
    Standard(StdVariable),
    Method(Box<dyn Method>),
    Function(Function),
    Custom(CustomVarWrapper),
}

impl Variable {
    pub fn str(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        return match self {
            Variable::Null() => Result::Ok("null".into()),
            Variable::Bool(val) => Result::Ok((if *val { "true" } else { "false" }).into()),
            Variable::String(val) => Result::Ok(val.clone()),
            Variable::Bigint(val) => Result::Ok(val.to_str_radix(10).into()),
            Variable::Decimal(val) => Result::Ok(val.to_string().into()),
            Variable::Type(val) => Result::Ok(val.to_string().into()),
            Variable::Standard(val) => val.str(runtime),
            Variable::Function(val) => Result::Ok(val.to_str(runtime)),
            _ => unimplemented!(),
        };
    }

    pub fn int(&self, runtime: &mut Runtime) -> Result<BigInt, ()> {
        return match self {
            Variable::Bool(val) => Result::Ok(if *val { 1 } else { 0 }.into()),
            Variable::Bigint(val) => Result::Ok(val.clone()),
            Variable::Decimal(val) => Result::Ok(val.to_integer()),
            Variable::Standard(val) => val.int(runtime),
            Variable::String(val) => BigInt::from_str(val).or(Result::Err(())),
            _ => unimplemented!(),
        };
    }

    pub fn to_bool(&self, runtime: &mut Runtime) -> Result<bool, ()> {
        return match self {
            Variable::Null() => Result::Ok(false),
            Variable::Bool(val) => Result::Ok(*val),
            Variable::String(val) => Result::Ok(!val.is_empty()),
            Variable::Bigint(val) => Result::Ok(val != &BigInt::zero()),
            Variable::Decimal(val) => Result::Ok(val != &BigRational::zero()),
            Variable::Type(_) => Result::Ok(true),
            Variable::Standard(val) => val.bool(runtime),
            Variable::Method(_) => Result::Ok(true),
            Variable::Function(_) => Result::Ok(true),
            Variable::Custom(_) => unimplemented!(),
        };
    }

    pub fn call(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            Variable::Standard(val) => val.call(args),
            Variable::Method(method) => method.call(args),
            Variable::Function(func) => func.call(args),
            Variable::Type(t) => t.push_create(args),
            _ => unimplemented!(),
        }
    }

    pub fn index(&self, index: Name) -> Variable {
        return match self {
            Variable::Standard(val) => val.index(index),
            Variable::Bigint(val) => {
                if let Name::Operator(o) = index {
                    get_operator(val, o)
                } else {
                    unimplemented!()
                }
            }
            Variable::String(val) => {
                if let Name::Operator(o) = index {
                    string_functions::get_operator(val, o)
                } else {
                    unimplemented!()
                }
            }
            Variable::Type(t) => t.index(index),
            _ => unimplemented!(),
        };
    }

    pub fn set(&self, index: StringVar, value: Variable, _runtime: &mut Runtime) {
        match self {
            Variable::Standard(val) => val.set(index, value),
            Variable::Custom(_) => unimplemented!(),
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
            Variable::Function(_) => unimplemented!(),
            Variable::Custom(_) => unimplemented!(),
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
            (Variable::Custom(_), Variable::Custom(_)) => unimplemented!(),
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
            Variable::Function(val) => {
                if let Variable::Function(o) = other {
                    val == o
                } else {
                    false
                }
            }
            Variable::Custom(_) => unimplemented!(),
        };
    }
}

impl Eq for Variable {}

impl Hash for &'static FileInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self, state);
    }
}

impl Function {
    fn call(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            Function::Standard(file_no, fn_no) => {
                args.1.push_stack(0, *fn_no as u16, args.0, *file_no)?;
                FnResult::Ok(())
            }
            Function::Native(func) => {
                args.1.push_native();
                let result = func(args.0, args.1);
                args.1.pop_native();
                result
            }
        }
    }

    fn to_str(&self, runtime: &mut Runtime) -> StringVar {
        match self {
            Function::Standard(file_no, fn_no) => runtime.get_fn_name(*file_no, *fn_no),
            Function::Native(func) => format!("fn@{}", *func as usize).into(),
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Standard(a1, a2), Function::Standard(b1, b2)) => a1 == b1 && a2 == b2,
            (Function::Native(x), Function::Native(y)) => *x as usize == *y as usize,
            _ => false,
        }
    }
}

impl Eq for Function {}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Function::Standard(a, b) => {
                a.hash(state);
                b.hash(state);
            }
            Function::Native(a) => {
                state.write_usize(*a as usize);
            }
        }
    }
}

impl From<BigInt> for Variable {
    fn from(x: BigInt) -> Self {
        Variable::Bigint(x)
    }
}

impl From<StdVariable> for Variable {
    fn from(x: StdVariable) -> Self {
        Variable::Standard(x)
    }
}

impl From<String> for Variable {
    fn from(x: String) -> Self {
        Variable::String(x.into())
    }
}

impl From<StringVar> for Variable {
    fn from(x: StringVar) -> Self {
        Variable::String(x)
    }
}

impl From<Type> for Variable {
    fn from(x: Type) -> Self {
        Variable::Type(x)
    }
}

impl From<Variable> for BigInt {
    fn from(var: Variable) -> Self {
        match var {
            Variable::Bigint(i) => i,
            Variable::Bool(b) => if b { 1 } else { 0 }.into(),
            _ => panic!("Attempted to turn a variable not a superclass of int into an int"),
        }
    }
}

impl From<Variable> for StringVar {
    fn from(var: Variable) -> Self {
        if let Variable::String(s) = var {
            s
        } else {
            panic!("Attempted to turn a variable not a superclass of str into a str")
        }
    }
}

impl From<Variable> for bool {
    fn from(var: Variable) -> Self {
        if let Variable::Bool(b) = var {
            b
        } else {
            panic!("Attempted to turn a variable not a superclass of bool into a bool")
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Standard(i, j) => f.debug_tuple("Standard").field(i).field(j).finish(),
            Function::Native(fn_) => f
                .debug_tuple("Native")
                .field(&format!("fn@{}", *fn_ as usize))
                .finish(),
        }
    }
}
