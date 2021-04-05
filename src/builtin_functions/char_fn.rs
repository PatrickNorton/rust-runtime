use crate::character;
use crate::int_var::IntVar;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use ascii::{AsciiString, ToAsciiChar};

pub fn op_fn(o: Operator) -> NativeMethod<char> {
    match o {
        Operator::Equals => eq,
        Operator::Int => int,
        Operator::Str => str,
        Operator::Repr => repr,
        x => unimplemented!("char.{}", x.name()),
    }
}

pub fn get_operator(this: char, o: Operator) -> Variable {
    let func = op_fn(o);
    StdMethod::new_native(this, func).into()
}

pub fn attr_fn(s: &str) -> NativeMethod<char> {
    match s {
        "upper" => upper,
        "lower" => lower,
        x => unimplemented!("char.{}", x),
    }
}

pub fn get_attribute(this: char, s: &str) -> Variable {
    let func = attr_fn(s);
    StdMethod::new_native(this, func).into()
}

fn eq(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    runtime.return_1(args.into_iter().any(|arg| char::from(arg) != this).into())
}

fn int(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1((this as u32).into())
}

fn str(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let str = match this.to_ascii_char() {
        Result::Ok(chr) => StringVar::from(AsciiString::from(vec![chr])),
        Result::Err(_) => StringVar::from(this.to_string()),
    };
    runtime.return_1(str.into())
}

fn repr(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result: StringVar = match this {
        '\'' => "c\"'\"".into(),
        x => character::repr(x).into(),
    };
    runtime.return_1(result.into())
}

fn upper(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_uppercase().next().unwrap_or(this).into())
}

fn lower(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_lowercase().next().unwrap_or(this).into())
}
