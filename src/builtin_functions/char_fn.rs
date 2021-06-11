use crate::character;
use crate::custom_types::bytes::LangBytes;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use ascii::{AsciiString, ToAsciiChar};
use std::array::IntoIter;
use std::rc::Rc;

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
        "isUpper" => is_upper,
        "isLower" => is_lower,
        "utf8Len" => utf8_len,
        "utf16Len" => utf16_len,
        "encodeUtf8" => encode_utf8,
        "encodeUtf16" => encode_utf16,
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
        '"' => "c'\"'".into(),
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

fn is_upper(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.is_uppercase().into())
}

fn is_lower(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.is_lowercase().into())
}

fn utf8_len(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.len_utf8().into())
}

fn utf16_len(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.len_utf16().into())
}

fn encode_utf8(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let mut result = [0; 4];
    let len = this.encode_utf8(&mut result).len();
    runtime.return_1(Rc::new(LangBytes::new(result[..len].into())).into())
}

fn encode_utf16(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let mut result = [0; 2];
    let len = this.encode_utf16(&mut result).len();
    runtime.return_1(
        Rc::new(LangBytes::new(
            result[..len]
                .iter()
                .flat_map(|x| IntoIter::new(x.to_le_bytes()))
                .collect(),
        ))
        .into(),
    )
}
