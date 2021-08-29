use crate::builtin_functions::Encoding;
use crate::custom_types::bytes::LangBytes;
use crate::custom_types::exceptions::value_error;
use crate::function::Function;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use crate::{character, first};
use ascii::{AsciiString, ToAsciiChar};
use num::ToPrimitive;
use std::fmt::Display;
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
        "encode" => encode,
        "isDigit" => is_digit,
        "isNumeric" => is_numeric,
        x => unimplemented!("char.{}", x),
    }
}

pub fn get_attribute(this: char, s: &str) -> Variable {
    let func = attr_fn(s);
    StdMethod::new_native(this, func).into()
}

pub fn static_attr(s: &str) -> Variable {
    let func = match s {
        "fromInt" => from_int,
        x => unimplemented!("str.{}", x),
    };
    Function::Native(func).into()
}

fn eq(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    runtime.return_1(args.into_iter().all(|arg| char::from(arg) == this).into())
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
        x => format!("c'{}'", character::repr(x)).into(),
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

fn is_digit(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let first = first(args).int(runtime)?;
    let radix = match first.to_u32() {
        Option::None => return base_err(first, runtime),
        Option::Some(x) => {
            if (0..=36).contains(&x) {
                x
            } else {
                return base_err(x, runtime);
            }
        }
    };
    runtime.return_1(this.is_digit(radix).into())
}

fn is_numeric(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.is_numeric().into())
}

fn encode(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let encoding = match Encoding::from_str(&first(args).str(runtime)?) {
        Result::Ok(x) => x,
        Result::Err(x) => {
            return runtime
                .throw_quick_native(value_error(), format!("{} is not a valid encoding", x))
        }
    };
    let bytes = match encoding {
        Encoding::Ascii => {
            if this.is_ascii() {
                vec![this as u32 as u8]
            } else {
                return runtime.throw_quick_native(
                    value_error(),
                    format!(
                        "Cannot convert to ascii: character {} (Unicode value {:x}) is not ASCII",
                        this, this as u32
                    ),
                );
            }
        }
        Encoding::Utf8 => {
            let mut result = [0; 4];
            let len = this.encode_utf8(&mut result).len();
            result[..len].into()
        }
        Encoding::Utf16Le => encode_utf_16(this, false),
        Encoding::Utf16Be => encode_utf_16(this, true),
        Encoding::Utf32Le => (this as u32).to_le_bytes().into(),
        Encoding::Utf32Be => (this as u32).to_be_bytes().into(),
    };
    runtime.return_1(Rc::new(LangBytes::new(bytes)).into())
}

#[inline]
fn encode_utf_16(value: char, big_end: bool) -> Vec<u8> {
    let mut result = [0; 2];
    let len = value.encode_utf16(&mut result).len();
    result[..len]
        .iter()
        .flat_map(|x| {
            if big_end {
                x.to_be_bytes()
            } else {
                x.to_le_bytes()
            }
        })
        .collect()
}

fn from_int(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let arg = first(args).int(runtime)?;
    let value = arg.to_u32().and_then(char::from_u32).map(From::from);
    runtime.return_1(value.into())
}

fn base_err<T: Display>(value: T, runtime: &mut Runtime) -> FnResult {
    runtime.throw_quick_native(
        value_error(),
        format!(
            "Invalid base: bases must be between 0 and 36, got {}",
            value
        ),
    )
}

#[cfg(test)]
mod test {
    use crate::builtin_functions::char_fn::{encode_utf_16, eq, int, lower, repr, upper};
    use crate::runtime::Runtime;
    use crate::string_var::StringVar;

    #[test]
    fn char_eq() {
        let v1 = Runtime::test(|runtime| eq('a', vec!['a'.into()], runtime));
        assert_eq!(v1, Result::Ok(true.into()));
        let v2 = Runtime::test(|runtime| eq('a', vec!['b'.into()], runtime));
        assert_eq!(v2, Result::Ok(false.into()));
        let v3 = Runtime::test(|runtime| eq('a', vec!['a'.into(), 'a'.into()], runtime));
        assert_eq!(v3, Result::Ok(true.into()));
        let v4 = Runtime::test(|runtime| eq('a', vec!['a'.into(), 'b'.into()], runtime));
        assert_eq!(v4, Result::Ok(false.into()));
    }

    #[test]
    fn char_int() {
        let v1 = Runtime::test(|runtime| int('a', vec![], runtime));
        assert_eq!(v1, Result::Ok(('a' as u32).into()));
    }

    #[test]
    fn char_repr() {
        let quote = Runtime::test(|runtime| repr('"', vec![], runtime));
        assert_eq!(quote, Result::Ok(StringVar::from("c'\"'").into()));
        let char = Runtime::test(|runtime| repr('a', vec![], runtime));
        assert_eq!(char, Result::Ok(StringVar::from("c'a'").into()));
    }

    #[test]
    fn char_upper() {
        let val = Runtime::test(|runtime| upper('a', vec![], runtime));
        assert_eq!(val, Result::Ok('A'.into()));
    }

    #[test]
    fn char_lower() {
        let val = Runtime::test(|runtime| lower('A', vec![], runtime));
        assert_eq!(val, Result::Ok('a'.into()));
    }

    #[test]
    fn encode_u16() {
        let encoded = encode_utf_16('á', false);
        assert_eq!(encoded, vec![0xe1, 0x00]);
    }
}
