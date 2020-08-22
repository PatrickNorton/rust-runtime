use crate::builtin_functions::{bool_fn, char_fn, dec_fn, int_fn, null_fn, string_fn, tuple_fn};
use crate::custom_var::CustomVarWrapper;
use crate::file_info::FileInfo;
use crate::function::Function;
use crate::int_var::IntVar;
use crate::lang_union::LangUnion;
use crate::looping;
use crate::method::Method;
use crate::name::Name;
use crate::operator::Operator;
use crate::option::LangOption;
use crate::quick_functions::quick_equals;
use crate::rational_var::RationalVar;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use crate::string_var::StringVar;
use crate::tuple::LangTuple;
use num::bigint::BigInt;
use num::traits::Zero;
use num::{BigRational, ToPrimitive};
use std::boxed::Box;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;
use std::str::FromStr;
use std::string::String;
use std::vec::Vec;

pub type FnResult = Result<(), ()>;

#[derive(Debug, Clone)]
pub enum Variable {
    Null(),
    Bool(bool),
    Bigint(IntVar),
    String(StringVar),
    Decimal(RationalVar),
    Char(char),
    Type(Type),
    Standard(StdVariable),
    Tuple(LangTuple),
    Method(Box<dyn Method>),
    Function(Function),
    Custom(CustomVarWrapper),
    Union(LangUnion),
    Option(LangOption),
}

impl Variable {
    pub fn str(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        match self {
            Variable::Null() => Result::Ok("null".into()),
            Variable::Bool(val) => Result::Ok((if *val { "true" } else { "false" }).into()),
            Variable::String(val) => Result::Ok(val.clone()),
            Variable::Bigint(val) => Result::Ok(val.to_string().into()),
            Variable::Decimal(val) => Result::Ok(val.to_string().into()),
            Variable::Char(val) => Result::Ok(val.to_string().into()),
            Variable::Type(val) => Result::Ok(val.str()),
            Variable::Standard(val) => val.str(runtime),
            Variable::Tuple(val) => val.str(runtime),
            Variable::Function(val) => Result::Ok(val.to_str(runtime)),
            Variable::Custom(val) => (**val).clone().str(runtime),
            Variable::Union(val) => val.str(runtime),
            Variable::Option(val) => val.str(runtime),
            Variable::Method(_) => unimplemented!(),
        }
    }

    pub fn repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        match self {
            Variable::Null() => Result::Ok("null".into()),
            Variable::Bool(val) => Result::Ok((if *val { "true" } else { "false" }).into()),
            Variable::String(val) => Result::Ok(format!("{:?}", val.as_str()).into()),
            Variable::Bigint(val) => Result::Ok(val.to_string().into()),
            Variable::Decimal(val) => Result::Ok(val.to_string().into()),
            Variable::Char(val) => Result::Ok(val.to_string().into()),
            Variable::Type(val) => Result::Ok(val.str()),
            Variable::Standard(val) => val.repr(runtime),
            Variable::Tuple(val) => val.repr(runtime),
            Variable::Function(val) => Result::Ok(val.to_str(runtime)),
            Variable::Custom(val) => (**val).clone().repr(runtime),
            Variable::Union(val) => val.repr(runtime),
            Variable::Option(val) => val.repr(runtime),
            Variable::Method(_) => unimplemented!(),
        }
    }

    pub fn int(&self, runtime: &mut Runtime) -> Result<IntVar, ()> {
        match self {
            Variable::Bool(val) => Result::Ok(if *val { 1 } else { 0 }.into()),
            Variable::Bigint(val) => Result::Ok(val.clone()),
            Variable::Decimal(val) => Result::Ok(val.to_integer().into()),
            Variable::Char(val) => Result::Ok((*val as u32).into()),
            Variable::Standard(val) => val.int(runtime),
            Variable::String(val) => Result::Ok(IntVar::from_str(val)?),
            Variable::Custom(val) => (**val).clone().int(runtime),
            Variable::Union(val) => val.int(runtime),
            _ => unimplemented!(),
        }
    }

    pub fn to_bool(&self, runtime: &mut Runtime) -> Result<bool, ()> {
        match self {
            Variable::Null() => Result::Ok(false),
            Variable::Bool(val) => Result::Ok(*val),
            Variable::String(val) => Result::Ok(!val.is_empty()),
            Variable::Bigint(val) => Result::Ok(!val.is_zero()),
            Variable::Decimal(val) => Result::Ok(val != &RationalVar::zero()),
            Variable::Char(val) => Result::Ok(val != &'\0'),
            Variable::Type(_) => Result::Ok(true),
            Variable::Standard(val) => val.bool(runtime),
            Variable::Tuple(val) => Result::Ok(!val.is_empty()),
            Variable::Method(_) => Result::Ok(true),
            Variable::Function(_) => Result::Ok(true),
            Variable::Custom(val) => (**val).clone().bool(runtime),
            Variable::Union(val) => val.bool(runtime),
            Variable::Option(val) => Result::Ok(val.is_some()),
        }
    }

    pub fn call(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            Variable::Standard(val) => val.call(args),
            Variable::Method(method) => method.call(args),
            Variable::Function(func) => func.call(args),
            Variable::Type(t) => t.push_create(args),
            Variable::Custom(val) => (**val).clone().call(args.0, args.1),
            Variable::Union(val) => val.call(args),
            _ => unimplemented!(),
        }
    }

    pub fn call_or_goto(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            Variable::Standard(val) => val.call_or_goto(args),
            Variable::Method(method) => method.call_or_goto(args),
            Variable::Function(func) => func.call_or_goto(args),
            Variable::Type(t) => t.push_create(args),
            Variable::Custom(val) => (**val).clone().call_or_goto(args.0, args.1),
            Variable::Union(val) => val.call_or_goto(args),
            _ => unimplemented!(),
        }
    }

    pub fn iter(&self, runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        match self {
            Variable::String(_) => todo!(),
            Variable::Type(_) => unimplemented!("Enum type iteration not completed yet"),
            Variable::Standard(val) => val.iter(runtime),
            Variable::Custom(val) => (**val).clone().iter(runtime),
            Variable::Union(val) => val.iter(runtime),
            _ => unimplemented!(),
        }
    }

    pub fn index(self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        Result::Ok(match self {
            Variable::Null() => {
                if let Name::Operator(o) = index {
                    null_fn::get_operator(o)
                } else {
                    unimplemented!()
                }
            }
            Variable::Standard(val) => val.index(index, runtime)?,
            Variable::Bool(val) => {
                if let Name::Operator(o) = index {
                    bool_fn::get_operator(val, o)
                } else {
                    unimplemented!()
                }
            }
            Variable::Bigint(val) => {
                if let Name::Operator(o) = index {
                    int_fn::get_operator(val, o)
                } else {
                    unimplemented!()
                }
            }
            Variable::String(val) => match index {
                Name::Operator(o) => string_fn::get_operator(val, o),
                Name::Attribute(s) => string_fn::get_attr(val, s),
            },
            Variable::Tuple(val) => match index {
                Name::Operator(o) => tuple_fn::get_operator(val, o),
                Name::Attribute(s) => tuple_fn::get_attr(val, s),
            },
            Variable::Decimal(val) => {
                if let Name::Operator(o) = index {
                    dec_fn::get_operator(val, o)
                } else {
                    unimplemented!()
                }
            }
            Variable::Char(val) => {
                if let Name::Operator(o) = index {
                    char_fn::get_operator(val, o)
                } else {
                    unimplemented!()
                }
            }
            Variable::Type(t) => t.index(index, runtime),
            Variable::Custom(val) => (*val).clone().get_attr(index),
            Variable::Union(val) => val.index(index, runtime)?,
            _ => unimplemented!(),
        })
    }

    pub fn set(&self, index: StringVar, value: Variable, runtime: &mut Runtime) -> FnResult {
        match self {
            Variable::Standard(val) => val.set(index, value, runtime)?,
            Variable::Custom(val) => (**val).clone().set(Name::Attribute(index), value),
            Variable::Type(val) => val.set(index, value, runtime),
            _ => unimplemented!(),
        }
        runtime.return_0()
    }

    pub fn get_type(&self) -> Type {
        match self {
            Variable::Null() => Type::Null,
            Variable::Bool(_) => Type::Bool,
            Variable::String(_) => Type::String,
            Variable::Bigint(_) => Type::Bigint,
            Variable::Decimal(_) => Type::Decimal,
            Variable::Char(_) => Type::Char,
            Variable::Type(_) => Type::Type,
            Variable::Method(_) => unimplemented!(),
            Variable::Standard(a) => a.get_type(),
            Variable::Tuple(_) => Type::Tuple,
            Variable::Function(_) => unimplemented!(),
            Variable::Custom(a) => (**a).clone().get_type(),
            Variable::Union(val) => val.get_type(),
            Variable::Option(_) => unimplemented!(),
        }
    }

    pub fn identical(&self, other: &Variable) -> bool {
        match (self, other) {
            (Variable::Null(), Variable::Null()) => true,
            (Variable::Bool(a), Variable::Bool(b)) => a == b,
            (Variable::String(a), Variable::String(b)) => a == b,
            (Variable::Bigint(a), Variable::Bigint(b)) => a == b,
            (Variable::Decimal(a), Variable::Decimal(b)) => a == b,
            (Variable::Char(a), Variable::Char(b)) => a == b,
            (Variable::Type(a), Variable::Type(b)) => a == b,
            (Variable::Standard(a), Variable::Standard(b)) => a.identical(b),
            (Variable::Tuple(a), Variable::Tuple(b)) => a.identical(b),
            (Variable::Method(a), Variable::Method(b)) => a == b,
            (Variable::Custom(a), Variable::Custom(b)) => a == b,
            (Variable::Union(a), Variable::Union(b)) => a == b,
            (Variable::Option(a), Variable::Option(b)) => a == b,
            _ => false,
        }
    }

    pub fn equals(&self, other: Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        quick_equals(self.clone(), other, runtime)?.to_bool(runtime)
    }

    pub fn is_type_of(&self, other: &Variable) -> bool {
        if let Variable::Type(t) = self {
            t.is_type_of(other)
        } else {
            false
        }
    }

    pub fn hash(&self, runtime: &mut Runtime) -> Result<usize, ()> {
        match self {
            Variable::Null() => Result::Ok(0),
            Variable::Bool(b) => Result::Ok(if *b { 0 } else { 1 }),
            Variable::Bigint(i) => {
                let max = IntVar::Big(Rc::new(BigInt::from(std::usize::MAX) + 1));
                let hash = i.clone() % max;
                Result::Ok(hash.to_usize().unwrap())
            }
            Variable::String(s) => {
                let mut result = 0;
                for c in s.chars() {
                    result += c as usize;
                }
                Result::Ok(result)
            }
            Variable::Decimal(d) => {
                let max = BigInt::from(std::usize::MAX) + 1;
                let hash: BigInt = d.to_integer() % &max;
                Result::Ok(hash.to_usize().unwrap())
            }
            Variable::Char(c) => Result::Ok(*c as usize),
            Variable::Type(_) => unimplemented!(),
            Variable::Standard(v) => {
                runtime.push_native();
                v.call_operator(Operator::Hash, Vec::new(), runtime)?;
                runtime.pop_native();
                Result::Ok(IntVar::from(runtime.pop_return()).to_usize().unwrap())
            }
            Variable::Tuple(t) => t.lang_hash(runtime),
            Variable::Method(_) => unimplemented!(),
            Variable::Function(_) => unimplemented!(),
            Variable::Custom(val) => {
                runtime.push_native();
                (**val)
                    .clone()
                    .call_op(Operator::Hash, Vec::new(), runtime)?;
                runtime.pop_native();
                Result::Ok(IntVar::from(runtime.pop_return()).to_usize().unwrap())
            }
            Variable::Union(val) => {
                val.call_operator(Operator::Hash, Vec::new(), runtime)?;
                Result::Ok(IntVar::from(runtime.pop_return()).to_usize().unwrap())
            }
            Variable::Option(val) => val
                .as_ref()
                .map_or_else(|| Result::Ok(0), |x| (**x).hash(runtime)),
        }
    }

    pub fn call_op(self, name: Operator, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        match self {
            Variable::Null() => runtime.call_copy_method(null_fn::op_fn(name), (), args),
            Variable::Bool(b) => match bool_fn::op_fn(name) {
                Option::Some(val) => runtime.call_copy_method(val, b, args),
                Option::None => {
                    runtime.call_native_method(int_fn::op_fn(name), &IntVar::from_bool(b), args)
                }
            },
            Variable::Bigint(b) => runtime.call_native_method(int_fn::op_fn(name), &b, args),
            Variable::String(s) => runtime.call_native_method(string_fn::op_fn(name), &s, args),
            Variable::Decimal(d) => runtime.call_native_method(dec_fn::op_fn(name), &d, args),
            Variable::Char(c) => runtime.call_copy_method(char_fn::op_fn(name), c, args),
            Variable::Type(_) => self
                .index(Name::Operator(name), runtime)?
                .call((args, runtime)),
            Variable::Standard(s) => s.call_operator(name, args, runtime),
            Variable::Tuple(t) => runtime.call_native_method(tuple_fn::op_fn(name), &t, args),
            Variable::Method(_) => self
                .index(Name::Operator(name), runtime)?
                .call((args, runtime)),
            Variable::Function(_) => self
                .index(Name::Operator(name), runtime)?
                .call((args, runtime)),
            Variable::Custom(c) => (*c).clone().call_op(name, args, runtime),
            Variable::Union(u) => u.call_operator(name, args, runtime),
            Variable::Option(o) => o.call_op(name, args, runtime),
        }
    }

    pub fn call_op_or_goto(
        self,
        name: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        match self {
            Variable::Standard(s) => s.call_op_or_goto(name, args, runtime),
            Variable::Method(_) => self
                .index(Name::Operator(name), runtime)?
                .call_or_goto((args, runtime)),
            Variable::Function(_) => self
                .index(Name::Operator(name), runtime)?
                .call_or_goto((args, runtime)),
            Variable::Custom(c) => (*c).clone().call_op_or_goto(name, args, runtime),
            _ => self.call_op(name, args, runtime),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Variable::Null() => true,
            Variable::Option(a) => a.is_none(),
            _ => false,
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Variable::Null() => 0,
            Variable::Bool(_) => todo!("Unique ids for bool"),
            Variable::Bigint(b) => match b {
                IntVar::Small(_) => todo!("Unique ids for small int"),
                IntVar::Big(b) => &**b as *const _ as usize,
            },
            Variable::String(s) => s.as_str() as *const str as *const () as usize,
            Variable::Decimal(d) => &**d as *const BigRational as usize,
            Variable::Char(_) => todo!("Unique ids for char"),
            Variable::Type(t) => t.id(),
            Variable::Standard(s) => s.var_ptr(),
            Variable::Tuple(t) => t.id(),
            Variable::Method(_) => todo!("Unique ids for method"),
            Variable::Function(f) => f.id(),
            Variable::Custom(c) => &**c as *const _ as usize,
            Variable::Union(u) => u.get_value().id(),
            Variable::Option(o) => o.as_ref().map_or(0, |x| x.id()),
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl Eq for Variable {}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Variable::Null() => 0.hash(state),
            Variable::Bool(b) => b.hash(state),
            Variable::Bigint(i) => i.hash(state),
            Variable::String(s) => s.hash(state),
            Variable::Decimal(d) => d.hash(state),
            Variable::Char(c) => c.hash(state),
            Variable::Type(t) => t.hash(state),
            Variable::Standard(s) => s.hash(state),
            Variable::Tuple(t) => t.hash(state),
            Variable::Method(m) => m.hash(state),
            Variable::Function(f) => f.hash(state),
            Variable::Custom(c) => c.hash(state),
            Variable::Union(u) => u.hash(state),
            Variable::Option(o) => o.hash(state),
        }
    }
}

impl Hash for &'static FileInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self, state);
    }
}

impl From<IntVar> for Variable {
    fn from(x: IntVar) -> Self {
        Variable::Bigint(x)
    }
}

impl From<RationalVar> for Variable {
    fn from(x: RationalVar) -> Self {
        Variable::Decimal(x)
    }
}

impl From<StdVariable> for Variable {
    fn from(x: StdVariable) -> Self {
        Variable::Standard(x)
    }
}

impl From<LangUnion> for Variable {
    fn from(x: LangUnion) -> Self {
        Variable::Union(x)
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

impl From<LangTuple> for Variable {
    fn from(x: LangTuple) -> Self {
        Variable::Tuple(x)
    }
}

impl From<bool> for Variable {
    fn from(x: bool) -> Self {
        Variable::Bool(x)
    }
}

impl From<char> for Variable {
    fn from(x: char) -> Self {
        Variable::Char(x)
    }
}

impl From<()> for Variable {
    fn from(_: ()) -> Self {
        Variable::Null()
    }
}

impl From<Variable> for IntVar {
    fn from(var: Variable) -> Self {
        match var {
            Variable::Bigint(i) => i,
            Variable::Bool(b) => if b { 1 } else { 0 }.into(),
            _ => panic!("Attempted to turn a variable not a superclass of int into an int"),
        }
    }
}

impl From<Variable> for RationalVar {
    fn from(var: Variable) -> Self {
        if let Variable::Decimal(d) = var {
            d
        } else {
            panic!("Attempted to turn a variable not a superclass of dec into a dec")
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

impl From<Variable> for LangTuple {
    fn from(var: Variable) -> Self {
        if let Variable::Tuple(t) = var {
            t
        } else {
            panic!("Attempted to turn a variable not a superclass of tuple into a tuple")
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

impl From<Variable> for char {
    fn from(var: Variable) -> Self {
        if let Variable::Char(b) = var {
            b
        } else {
            panic!("Attempted to turn a variable not a superclass of char into a char")
        }
    }
}

impl From<Variable> for looping::Iterator {
    fn from(var: Variable) -> Self {
        match var {
            Variable::Custom(var) => (*var).clone().into_iter(),
            Variable::Standard(var) => looping::Iterator::NonNative(var),
            _ => unimplemented!(),
        }
    }
}

impl Default for Variable {
    fn default() -> Self {
        Variable::Null()
    }
}

pub(crate) trait FromBool {
    fn from_bool(x: bool) -> Self;
}

impl FromBool for BigInt {
    fn from_bool(x: bool) -> Self {
        if x { 1u8 } else { 0u8 }.into()
    }
}

impl FromBool for IntVar {
    fn from_bool(x: bool) -> Self {
        if x { 1u8 } else { 0u8 }.into()
    }
}
