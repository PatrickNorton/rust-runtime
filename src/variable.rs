use crate::builtin_functions::{
    bool_fn, char_fn, dec_fn, int_fn, null_fn, option_fn, string_fn, tuple_fn,
};
use crate::custom_var::CustomVarWrapper;
use crate::file_info::FileInfo;
use crate::function::Function;
use crate::int_var::IntVar;
use crate::lang_union::LangUnion;
use crate::looping;
use crate::method::Method;
use crate::name::Name;
use crate::operator::Operator;
use crate::quick_functions::quick_equals;
use crate::rational_var::RationalVar;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use crate::string_var::StringVar;
use crate::tuple::LangTuple;
use crate::var_impls::FromBool;
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
use std::vec::Vec;

pub type FnResult = Result<(), ()>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Variable {
    Normal(InnerVar),
    Option(usize, Option<InnerVar>),
}

#[derive(Debug, Clone)]
pub enum InnerVar {
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
}

pub struct OptionVar(pub usize, pub Option<InnerVar>);

impl Variable {
    pub fn null() -> Variable {
        Variable::default()
    }

    pub fn str(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        match self {
            Variable::Normal(var) => var.str(runtime),
            Variable::Option(i, val) => option_fn::str(i, val, runtime),
        }
    }

    pub fn repr(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        match self {
            Variable::Normal(var) => var.repr(runtime),
            Variable::Option(i, val) => option_fn::repr(i, val, runtime),
        }
    }

    pub fn int(self, runtime: &mut Runtime) -> Result<IntVar, ()> {
        match self {
            Variable::Normal(var) => var.int(runtime),
            Variable::Option(_, _) => unimplemented!(),
        }
    }

    pub fn into_bool(self, runtime: &mut Runtime) -> Result<bool, ()> {
        match self {
            Variable::Normal(var) => var.into_bool(runtime),
            Variable::Option(i, val) => Result::Ok(i > 1 || val.is_some()),
        }
    }

    pub fn call(self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            Variable::Normal(var) => var.call(args),
            Variable::Option(i, val) => unimplemented!(
                "{}()\n{}",
                option_fn::type_of(i, val.as_ref()).str(),
                args.1.stack_frames()
            ),
        }
    }

    pub fn call_or_goto(self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            Variable::Normal(var) => var.call_or_goto(args),
            Variable::Option(i, val) => unimplemented!(
                "{}()\n{}",
                option_fn::type_of(i, val.as_ref()).str(),
                args.1.stack_frames()
            ),
        }
    }

    pub fn iter(self, runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        match self {
            Variable::Normal(var) => var.iter(runtime),
            Variable::Option(i, val) => unimplemented!(
                "{}()\n{}",
                option_fn::type_of(i, val.as_ref()).str(),
                runtime.stack_frames()
            ),
        }
    }

    pub fn index(self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        match self {
            Variable::Normal(var) => var.index(index, runtime),
            Variable::Option(i, val) => Result::Ok(option_fn::index(i, val, index)),
        }
    }

    pub fn set(self, index: &str, value: Variable, runtime: &mut Runtime) -> FnResult {
        match self {
            Variable::Normal(var) => var.set(index, value, runtime)?,
            Variable::Option(_, _) => unimplemented!(),
        }
        runtime.return_0()
    }

    pub fn get_type(&self) -> Type {
        match self {
            Variable::Normal(val) => val.get_type(),
            Variable::Option(i, val) => option_fn::type_of(*i, val.as_ref()),
        }
    }

    pub fn identical(&self, other: &Variable) -> bool {
        match (self, other) {
            (Variable::Normal(a), Variable::Normal(b)) => a.identical(b),
            (Variable::Option(a1, a2), Variable::Option(b1, b2)) => a1 == b1 && a2 == b2,
            _ => false,
        }
    }

    pub fn equals(self, other: Variable, runtime: &mut Runtime) -> Result<bool, ()> {
        quick_equals(self, other, runtime)?.into_bool(runtime)
    }

    pub fn is_type_of(&self, other: &Variable, runtime: &Runtime) -> bool {
        if let Variable::Normal(InnerVar::Type(t)) = self {
            t.is_type_of(other, runtime)
        } else {
            false
        }
    }

    pub fn hash(self, runtime: &mut Runtime) -> Result<usize, ()> {
        match self {
            Variable::Normal(var) => var.hash(runtime),
            Variable::Option(_, val) => val.map_or_else(|| Result::Ok(0), |x| x.hash(runtime)),
        }
    }

    pub fn call_op(self, name: Operator, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        match self {
            Variable::Normal(var) => var.call_op(name, args, runtime),
            Variable::Option(i, var) => option_fn::call_op(i, var, name, args, runtime),
        }
    }

    pub fn call_op_or_goto(
        self,
        name: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        match self {
            Variable::Normal(var) => var.call_op_or_goto(name, args, runtime),
            Variable::Option(i, var) => option_fn::call_op(i, var, name, args, runtime),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Variable::Normal(var) => var.is_null(),
            Variable::Option(i, var) => *i == 1 && var.is_none(),
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Variable::Normal(var) => var.id(),
            Variable::Option(_, val) => val.as_ref().map_or(0, |x| x.id()),
        }
    }

    pub fn as_int(&self) -> Option<&IntVar> {
        match self {
            Variable::Normal(InnerVar::Bigint(x)) => Option::Some(x),
            _ => Option::None,
        }
    }
}

impl InnerVar {
    pub fn str(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        match self {
            InnerVar::Null() => Result::Ok("null".into()),
            InnerVar::Bool(val) => Result::Ok((if val { "true" } else { "false" }).into()),
            InnerVar::String(val) => Result::Ok(val),
            InnerVar::Bigint(val) => Result::Ok(val.to_string().into()),
            InnerVar::Decimal(val) => Result::Ok(val.to_string().into()),
            InnerVar::Char(val) => Result::Ok(val.to_string().into()),
            InnerVar::Type(val) => Result::Ok(val.str()),
            InnerVar::Standard(val) => val.str(runtime),
            InnerVar::Tuple(val) => val.str(runtime),
            InnerVar::Function(val) => Result::Ok(val.to_str(runtime)),
            InnerVar::Custom(val) => val.into_inner().str(runtime),
            InnerVar::Union(val) => val.str(runtime),
            InnerVar::Method(_) => unimplemented!(),
        }
    }

    pub fn repr(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        match self {
            InnerVar::Null() => Result::Ok("null".into()),
            InnerVar::Bool(val) => Result::Ok((if val { "true" } else { "false" }).into()),
            InnerVar::String(val) => Result::Ok(val.repr()),
            InnerVar::Bigint(val) => Result::Ok(val.to_string().into()),
            InnerVar::Decimal(val) => Result::Ok(val.to_string().into()),
            InnerVar::Char(val) => Result::Ok(val.to_string().into()),
            InnerVar::Type(val) => Result::Ok(val.str()),
            InnerVar::Standard(val) => val.repr(runtime),
            InnerVar::Tuple(val) => val.repr(runtime),
            InnerVar::Function(val) => Result::Ok(val.to_str(runtime)),
            InnerVar::Custom(val) => val.into_inner().repr(runtime),
            InnerVar::Union(val) => val.repr(runtime),
            InnerVar::Method(_) => unimplemented!(),
        }
    }

    pub fn int(self, runtime: &mut Runtime) -> Result<IntVar, ()> {
        match self {
            InnerVar::Bool(val) => Result::Ok(if val { 1 } else { 0 }.into()),
            InnerVar::Bigint(val) => Result::Ok(val),
            InnerVar::Decimal(val) => Result::Ok(val.to_integer().into()),
            InnerVar::Char(val) => Result::Ok((val as u32).into()),
            InnerVar::Standard(val) => val.int(runtime),
            InnerVar::String(val) => Result::Ok(IntVar::from_str(val.as_str())?),
            InnerVar::Custom(val) => val.into_inner().int(runtime),
            InnerVar::Union(val) => val.int(runtime),
            _ => unimplemented!(),
        }
    }

    pub fn into_bool(self, runtime: &mut Runtime) -> Result<bool, ()> {
        match self {
            InnerVar::Null() => Result::Ok(false),
            InnerVar::Bool(val) => Result::Ok(val),
            InnerVar::String(val) => Result::Ok(!val.is_empty()),
            InnerVar::Bigint(val) => Result::Ok(!val.is_zero()),
            InnerVar::Decimal(val) => Result::Ok(!val.is_zero()),
            InnerVar::Char(val) => Result::Ok(val != '\0'),
            InnerVar::Type(_) => Result::Ok(true),
            InnerVar::Standard(val) => val.bool(runtime),
            InnerVar::Tuple(val) => Result::Ok(!val.is_empty()),
            InnerVar::Method(_) => Result::Ok(true),
            InnerVar::Function(_) => Result::Ok(true),
            InnerVar::Custom(val) => val.into_inner().bool(runtime),
            InnerVar::Union(val) => val.bool(runtime),
        }
    }

    pub fn call(self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            InnerVar::Standard(val) => val.call(args),
            InnerVar::Method(method) => method.call(args),
            InnerVar::Function(func) => func.call(args),
            InnerVar::Type(t) => t.push_create(args),
            InnerVar::Custom(val) => val.into_inner().call(args.0, args.1),
            InnerVar::Union(val) => val.call(args),
            x => unimplemented!("{}()\n{}", x.get_type().str(), args.1.stack_frames()),
        }
    }

    pub fn call_or_goto(self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        match self {
            InnerVar::Standard(val) => val.call_or_goto(args),
            InnerVar::Method(method) => method.call_or_goto(args),
            InnerVar::Function(func) => func.call_or_goto(args),
            InnerVar::Type(t) => t.push_create(args),
            InnerVar::Custom(val) => val.into_inner().call_or_goto(args.0, args.1),
            InnerVar::Union(val) => val.call_or_goto(args),
            x => unimplemented!("{}()\n{}", x.get_type().str(), args.1.stack_frames()),
        }
    }

    pub fn iter(self, runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        match self {
            InnerVar::String(s) => Result::Ok(string_fn::iter(s).into()),
            InnerVar::Type(_) => unimplemented!("Enum type iteration not completed yet"),
            InnerVar::Standard(val) => val.iter(runtime),
            InnerVar::Custom(val) => val.into_inner().iter(runtime),
            InnerVar::Union(val) => val.iter(runtime),
            x => unimplemented!("{}.iter()\n{}", x.get_type().str(), runtime.stack_frames()),
        }
    }

    pub fn index(self, index: Name, runtime: &mut Runtime) -> Result<Variable, ()> {
        Result::Ok(match self {
            InnerVar::Null() => {
                if let Name::Operator(o) = index {
                    null_fn::get_operator(o)
                } else {
                    unimplemented!("null.{}\n{}", index.as_str(), runtime.stack_frames())
                }
            }
            InnerVar::Standard(val) => val.index(index, runtime)?,
            InnerVar::Bool(val) => {
                if let Name::Operator(o) = index {
                    bool_fn::get_operator(val, o)
                } else {
                    unimplemented!("bool.{}\n{}", index.as_str(), runtime.stack_frames())
                }
            }
            InnerVar::Bigint(val) => {
                if let Name::Operator(o) = index {
                    int_fn::get_operator(val, o)
                } else {
                    unimplemented!("int.{}\n{}", index.as_str(), runtime.stack_frames())
                }
            }
            InnerVar::String(val) => match index {
                Name::Operator(o) => string_fn::get_operator(val, o),
                Name::Attribute(s) => string_fn::get_attr(val, s),
            },
            InnerVar::Tuple(val) => match index {
                Name::Operator(o) => tuple_fn::get_operator(val, o),
                Name::Attribute(s) => tuple_fn::get_attr(val, s),
            },
            InnerVar::Decimal(val) => {
                if let Name::Operator(o) = index {
                    dec_fn::get_operator(val, o)
                } else {
                    unimplemented!("dec.{}\n{}", index.as_str(), runtime.stack_frames())
                }
            }
            InnerVar::Char(val) => {
                if let Name::Operator(o) = index {
                    char_fn::get_operator(val, o)
                } else {
                    unimplemented!("char.{}\n{}", index.as_str(), runtime.stack_frames())
                }
            }
            InnerVar::Type(t) => t.index(index, runtime),
            InnerVar::Custom(val) => val.into_inner().get_attr(index),
            InnerVar::Union(val) => val.index(index, runtime)?,
            InnerVar::Method(m) => {
                if index == Name::Operator(Operator::Call) {
                    m.into()
                } else {
                    unimplemented!("method.{}", index.as_str())
                }
            }
            InnerVar::Function(f) => {
                if index == Name::Operator(Operator::Call) {
                    f.into()
                } else {
                    unimplemented!("function.{}", index.as_str())
                }
            }
        })
    }

    pub fn set(self, index: &str, value: Variable, runtime: &mut Runtime) -> FnResult {
        match self {
            InnerVar::Standard(val) => val.set(index, value, runtime)?,
            InnerVar::Custom(val) => val.into_inner().set(Name::Attribute(index), value),
            InnerVar::Type(val) => val.set(index, value, runtime),
            _ => unimplemented!(),
        }
        runtime.return_0()
    }

    pub fn get_type(&self) -> Type {
        match self {
            InnerVar::Null() => Type::Null,
            InnerVar::Bool(_) => Type::Bool,
            InnerVar::String(_) => Type::String,
            InnerVar::Bigint(_) => Type::Bigint,
            InnerVar::Decimal(_) => Type::Decimal,
            InnerVar::Char(_) => Type::Char,
            InnerVar::Type(_) => Type::Type,
            InnerVar::Method(_) => unimplemented!(),
            InnerVar::Standard(a) => a.get_type(),
            InnerVar::Tuple(_) => Type::Tuple,
            InnerVar::Function(_) => unimplemented!(),
            InnerVar::Custom(a) => a.get_type(),
            InnerVar::Union(val) => val.get_type(),
        }
    }

    pub fn identical(&self, other: &InnerVar) -> bool {
        match (self, other) {
            (InnerVar::Null(), InnerVar::Null()) => true,
            (InnerVar::Bool(a), InnerVar::Bool(b)) => a == b,
            (InnerVar::String(a), InnerVar::String(b)) => a == b,
            (InnerVar::Bigint(a), InnerVar::Bigint(b)) => a == b,
            (InnerVar::Decimal(a), InnerVar::Decimal(b)) => a == b,
            (InnerVar::Char(a), InnerVar::Char(b)) => a == b,
            (InnerVar::Type(a), InnerVar::Type(b)) => a == b,
            (InnerVar::Standard(a), InnerVar::Standard(b)) => a.identical(b),
            (InnerVar::Tuple(a), InnerVar::Tuple(b)) => a.identical(b),
            (InnerVar::Method(a), InnerVar::Method(b)) => a == b,
            (InnerVar::Custom(a), InnerVar::Custom(b)) => a == b,
            (InnerVar::Union(a), InnerVar::Union(b)) => a == b,
            _ => false,
        }
    }

    pub fn hash(self, runtime: &mut Runtime) -> Result<usize, ()> {
        match self {
            InnerVar::Null() => Result::Ok(0),
            InnerVar::Bool(b) => Result::Ok(if b { 0 } else { 1 }),
            InnerVar::Bigint(i) => {
                let max = IntVar::Big(Rc::new(BigInt::from(usize::MAX) + 1));
                let hash = i % max;
                Result::Ok(hash.to_usize().unwrap())
            }
            InnerVar::String(s) => {
                let mut result = 0;
                for c in s.chars() {
                    result += c as usize;
                }
                Result::Ok(result)
            }
            InnerVar::Decimal(d) => {
                let max = BigInt::from(usize::MAX) + 1;
                let hash: BigInt = d.to_integer() % &max;
                Result::Ok(hash.to_usize().unwrap())
            }
            InnerVar::Char(c) => Result::Ok(c as usize),
            InnerVar::Type(_) => unimplemented!(),
            InnerVar::Standard(v) => {
                runtime.push_native();
                v.call_operator(Operator::Hash, Vec::new(), runtime)?;
                runtime.pop_native();
                Result::Ok(IntVar::from(runtime.pop_return()).to_usize().unwrap())
            }
            InnerVar::Tuple(t) => t.lang_hash(runtime),
            InnerVar::Method(_) => unimplemented!(),
            InnerVar::Function(_) => unimplemented!(),
            InnerVar::Custom(val) => {
                runtime.push_native();
                val.into_inner()
                    .call_op(Operator::Hash, Vec::new(), runtime)?;
                runtime.pop_native();
                Result::Ok(IntVar::from(runtime.pop_return()).to_usize().unwrap())
            }
            InnerVar::Union(val) => {
                val.call_operator(Operator::Hash, Vec::new(), runtime)?;
                Result::Ok(IntVar::from(runtime.pop_return()).to_usize().unwrap())
            }
        }
    }

    pub fn call_op(self, name: Operator, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        match self {
            InnerVar::Null() => runtime.call_native_method(null_fn::op_fn(name), (), args),
            InnerVar::Bool(b) => match bool_fn::op_fn(name) {
                Option::Some(val) => runtime.call_native_method(val, b, args),
                Option::None => {
                    runtime.call_native_method(int_fn::op_fn(name), IntVar::from_bool(b), args)
                }
            },
            InnerVar::Bigint(b) => runtime.call_native_method(int_fn::op_fn(name), b, args),
            InnerVar::String(s) => runtime.call_native_method(string_fn::op_fn(name), s, args),
            InnerVar::Decimal(d) => runtime.call_native_method(dec_fn::op_fn(name), d, args),
            InnerVar::Char(c) => runtime.call_native_method(char_fn::op_fn(name), c, args),
            InnerVar::Type(_) => self
                .index(Name::Operator(name), runtime)?
                .call((args, runtime)),
            InnerVar::Standard(s) => s.call_operator(name, args, runtime),
            InnerVar::Tuple(t) => runtime.call_native_method(tuple_fn::op_fn(name), t, args),
            InnerVar::Method(m) => {
                if name == Operator::Call {
                    m.call((args, runtime))
                } else {
                    unimplemented!("method.{}", name.name())
                }
            }
            InnerVar::Function(f) => {
                if name == Operator::Call {
                    f.call((args, runtime))
                } else {
                    unimplemented!("function.{}", name.name())
                }
            }
            InnerVar::Custom(c) => c.into_inner().call_op(name, args, runtime),
            InnerVar::Union(u) => u.call_operator(name, args, runtime),
        }
    }

    pub fn call_op_or_goto(
        self,
        name: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        match self {
            InnerVar::Standard(s) => s.call_op_or_goto(name, args, runtime),
            InnerVar::Method(m) => {
                if name == Operator::Call {
                    m.call_or_goto((args, runtime))
                } else {
                    unimplemented!("method.{}", name.name())
                }
            }
            InnerVar::Function(f) => {
                if name == Operator::Call {
                    f.call_or_goto((args, runtime))
                } else {
                    unimplemented!("function.{}", name.name())
                }
            }
            InnerVar::Custom(c) => c.into_inner().call_op_or_goto(name, args, runtime),
            _ => self.call_op(name, args, runtime),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, InnerVar::Null())
    }

    pub fn id(&self) -> usize {
        match self {
            InnerVar::Null() => 0,
            InnerVar::Bool(_) => todo!("Unique ids for bool"),
            InnerVar::Bigint(b) => match b {
                IntVar::Small(_) => todo!("Unique ids for small int"),
                IntVar::Big(b) => &**b as *const _ as usize,
            },
            InnerVar::String(s) => s.as_str() as *const str as *const () as usize,
            InnerVar::Decimal(d) => &**d as *const BigRational as usize,
            InnerVar::Char(_) => todo!("Unique ids for char"),
            InnerVar::Type(t) => t.id(),
            InnerVar::Standard(s) => s.var_ptr(),
            InnerVar::Tuple(t) => t.id(),
            InnerVar::Method(_) => todo!("Unique ids for method"),
            InnerVar::Function(f) => f.id(),
            InnerVar::Custom(c) => &**c as *const _ as usize,
            InnerVar::Union(u) => u.get_value().id(),
        }
    }
}

impl PartialEq for InnerVar {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl Eq for InnerVar {}

impl Hash for InnerVar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            InnerVar::Null() => 0.hash(state),
            InnerVar::Bool(b) => b.hash(state),
            InnerVar::Bigint(i) => i.hash(state),
            InnerVar::String(s) => s.hash(state),
            InnerVar::Decimal(d) => d.hash(state),
            InnerVar::Char(c) => c.hash(state),
            InnerVar::Type(t) => t.hash(state),
            InnerVar::Standard(s) => s.hash(state),
            InnerVar::Tuple(t) => t.hash(state),
            InnerVar::Method(m) => m.hash(state),
            InnerVar::Function(f) => f.hash(state),
            InnerVar::Custom(c) => c.hash(state),
            InnerVar::Union(u) => u.hash(state),
        }
    }
}

impl Hash for &'static FileInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self, state);
    }
}
