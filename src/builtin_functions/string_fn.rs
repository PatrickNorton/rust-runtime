use crate::custom_types::bytes::LangBytes;
use crate::custom_types::exceptions::{arithmetic_error, index_error, value_error};
use crate::custom_types::list::List;
use crate::custom_var::CustomVar;
use crate::function::Function;
use crate::int_var::IntVar;
use crate::looping::{IterResult, NativeIterator};
use crate::method::{InnerMethod, NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::{AsciiVar, MaybeAscii, StrVar, StringVar};
use crate::variable::{FnResult, Variable};
use ascii::{AsAsciiStr, AsciiChar};
use downcast_rs::Downcast;
use num::{BigInt, Signed, ToPrimitive};
use std::any::Any;
use std::cell::Cell;
use std::fmt::Debug;
use std::mem::{replace, take};
use std::rc::Rc;
use std::str::{from_utf8_unchecked, FromStr};

pub fn op_fn(o: Operator) -> NativeMethod<StringVar> {
    match o {
        Operator::Add => add,
        Operator::Multiply => multiply,
        Operator::Bool => bool,
        Operator::Int => int,
        Operator::Str => str,
        Operator::Repr => repr,
        Operator::GetAttr => index,
        Operator::Iter => iter,
        Operator::Reversed => reversed,
        _ => unimplemented!("str.{}", o.name()),
    }
}

pub fn get_operator(this: StringVar, o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

pub fn get_attr(this: StringVar, s: StringVar) -> Variable {
    let func = match s.as_str() {
        "length" => return Variable::Bigint(this.chars().count().into()),
        "upper" => upper,
        "lower" => lower,
        "join" => join,
        "joinAll" => join_all,
        "startsWith" => starts_with,
        "endsWith" => ends_with,
        "split" => split,
        "splitlines" => split_lines,
        "indexOf" => index_of,
        "chars" => return chars(&this),
        "encode" => encode,
        "asInt" => as_int,
        _ => unimplemented!(),
    };
    Variable::Method(StdMethod::new_native(this, func))
}

pub fn static_attr(s: StringVar) -> Variable {
    let func = match s.as_str() {
        "fromChars" => from_chars,
        _ => unimplemented!(),
    };
    Variable::Function(Function::Native(func))
}

fn add(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let result = args.into_iter().fold(this.to_string(), |acc, arg| {
        acc + StringVar::from(arg).as_ref()
    });
    runtime.return_1(Variable::String(result.into()))
}

fn multiply(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if this.is_empty() {
        return runtime.return_1(this.clone().into());
    }
    let mut result: String = this.to_string();
    for arg in args {
        let big_val = IntVar::from(arg);
        match big_val.to_usize() {
            Option::Some(val) => match val.checked_mul(result.len()) {
                Option::Some(_) => result = result.repeat(val),
                Option::None => {
                    return runtime.throw_quick(arithmetic_error(), overflow_exc(val, result.len()))
                }
            },
            Option::None => return runtime.throw_quick(arithmetic_error(), mul_exc(big_val)),
        }
    }
    runtime.return_1(Variable::String(result.into()))
}

fn mul_exc(big_val: IntVar) -> StringVar {
    format!(
        "Too many string repetitions: max number of shifts \
            for a non-empty string is {}, attempted to shift by {}",
        usize::MAX,
        big_val,
    )
    .into()
}

fn overflow_exc(val: usize, len: usize) -> StringVar {
    format!(
        "Too many string repetitions: maximum string length is {}, \
        but repetition would produce str of length {}",
        usize::MAX,
        BigInt::from(val) * len
    )
    .into()
}

fn bool(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Bool(this.is_empty()))
}

fn int(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    match IntVar::from_str(this) {
        Ok(val) => runtime.push(Variable::Bigint(val)),
        Err(_) => {
            return runtime.throw_quick(
                value_error(),
                format!(
                    "Invalid input for int(str): {:?} is not a valid base-10 integer",
                    this.as_str()
                )
                .into(),
            )
        }
    }
    runtime.return_0()
}

fn str(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String(this.clone()))
}

fn repr(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String(format!("{:?}", this.as_str()).into()))
}

fn index(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let big_index = IntVar::from(take(&mut args[0]));
    let proper_index = if big_index.is_negative() {
        &big_index + &this.char_len().into()
    } else {
        big_index.clone()
    };
    let index = match proper_index.to_usize() {
        Option::Some(val) => val,
        Option::None => {
            return runtime.throw_quick(index_error(), bounds_msg(big_index, this.char_len()))
        }
    };
    match this.char_at(index) {
        Option::None => runtime.throw_quick(index_error(), bounds_msg(big_index, this.char_len())),
        Option::Some(value) => runtime.return_1(value.into()),
    }
}

fn bounds_msg(big_index: IntVar, char_len: usize) -> StringVar {
    format!(
        "Index {} out of bounds for str of length {}",
        big_index, char_len
    )
    .into()
}

fn iter(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(match this.clone().split_ascii() {
        Result::Ok(a) => Rc::new(AsciiIter::new(a)).into(),
        Result::Err(s) => Rc::new(StringIter::new(s)).into(),
    })
}

fn reversed(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.chars().rev().collect::<String>().into())
}

fn upper(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_uppercase().into())
}

fn lower(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_lowercase().into())
}

fn join(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let mut is_first = true;
    let mut result = String::new();
    let iter = replace(&mut args[0], Variable::Null()).iter(runtime)?;
    while let Option::Some(val) = iter.next(runtime)? {
        if !is_first {
            result += this;
        }
        is_first = false;
        result += val.str(runtime)?.as_str();
    }
    runtime.return_1(result.into())
}

fn join_all(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut result = String::with_capacity(this.char_len() * args.len());
    let len = args.len();
    for (i, val) in args.into_iter().enumerate() {
        result += val.str(runtime)?.as_str();
        if i + 1 < len {
            result += this;
        }
    }
    runtime.return_1(result.into())
}

fn starts_with(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 2);
    let val = StringVar::from(replace(&mut args[0], Variable::Null()));
    let index = IntVar::from(replace(&mut args[1], Variable::Null()));
    if index < this.char_len().into() {
        let usize_index = index
            .to_usize()
            .expect("String index believed to be less than a usize, but to_usize failed");
        if usize_index == 0 {
            runtime.return_1(this.starts_with(val.as_str()).into())
        } else {
            let mut chars = this.chars();
            chars.nth(usize_index - 1);
            runtime.return_1(chars.as_str().starts_with(val.as_str()).into())
        }
    } else {
        runtime.throw_quick(index_error(), "".into())
    }
}

fn ends_with(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let val = StringVar::from(take(&mut args[0]));
    runtime.return_1(this.ends_with(&*val).into())
}

fn split(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 2);
    let pat = StringVar::from(replace(&mut args[0], Variable::Null()));
    let opt_count = replace(&mut args[1], Variable::Null());
    if opt_count.is_null() {
        let result = List::from_values(
            Type::String,
            this.split(&*pat)
                .map(|a| StringVar::from(a.to_owned()))
                .map(Variable::from)
                .collect(),
        );
        runtime.return_1(result.into())
    } else {
        let val = IntVar::from(opt_count);
        let iterator = this
            .split(&*pat)
            .map(|a| StringVar::from(a.to_owned()))
            .map(Variable::from);
        let result = List::from_values(
            Type::String,
            match val.to_usize() {
                Option::Some(count) => iterator.take(count).collect(),
                Option::None => iterator.collect(),
            },
        );
        runtime.return_1(result.into())
    }
}

fn split_lines(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = List::from_values(
        Type::String,
        this.lines()
            .map(|a| StringVar::from(a.to_owned()))
            .map(Variable::from)
            .collect(),
    );
    runtime.return_1(result.into())
}

fn chars(this: &StringVar) -> Variable {
    List::from_values(Type::Char, this.chars().map(Variable::Char).collect()).into()
}

fn from_chars(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let mut result = String::new();
    let chars = take(&mut args[0]).iter(runtime)?;
    while let Option::Some(val) = chars.next(runtime)? {
        result.push(val.into());
    }
    runtime.return_1(result.into())
}

fn index_of(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let chr: char = take(&mut args[0]).into();
    let index = match this.as_maybe_ascii() {
        MaybeAscii::Standard(s) => s
            .chars()
            .enumerate()
            .find(|(_, c)| *c == chr)
            .map(|(i, _)| i),
        MaybeAscii::Ascii(a) => a
            .chars()
            .enumerate()
            .find(|(_, c)| *c == chr)
            .map(|(i, _)| i),
    };
    runtime.return_1(index.map(IntVar::from).map(Variable::from).into())
}

fn encode(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    // #![feature(array_value_iter)] will make this so much easier...
    let byte_val = match take(&mut args[0]).str(runtime)?.as_str() {
        "utf-8" => this.as_bytes().to_vec(),
        "utf-16" => this
            .encode_utf16()
            .flat_map(|x| x.to_le_bytes().to_vec())
            .collect(),
        "utf-16be" => this
            .encode_utf16()
            .flat_map(|x| x.to_be_bytes().to_vec())
            .collect(),
        "utf-32" => this
            .chars()
            .flat_map(|x| (x as u32).to_le_bytes().to_vec())
            .collect(),
        "utf-32be" => this
            .chars()
            .flat_map(|x| (x as u32).to_be_bytes().to_vec())
            .collect(),
        x => {
            return runtime.throw_quick(
                value_error(),
                format!("{} is not a valid encoding", x).into(),
            )
        }
    };
    runtime.return_1(Rc::new(LangBytes::new(byte_val.to_vec())).into())
}

fn as_int(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(IntVar::from_str(this).ok().map(Variable::from).into())
}

pub trait StrIter: Debug + Any + Downcast {
    fn next_fn(&self) -> Option<Variable>;

    fn next_func(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.next_fn().into())
    }
}

#[derive(Debug)]
pub struct StringIter {
    index: Cell<usize>,
    val: StrVar,
}

#[derive(Debug)]
pub struct AsciiIter {
    index: Cell<usize>,
    val: AsciiVar,
}

impl StringIter {
    fn new(val: StrVar) -> StringIter {
        StringIter {
            val,
            index: Cell::new(0),
        }
    }
}

impl StrIter for StringIter {
    fn next_fn(&self) -> Option<Variable> {
        let len = self.val.chars().count();
        if self.index.get() < len {
            let mut indices = unsafe {
                // We know this is safe b/c:
                // * The slice comes from a valid str, therefore, no invalid UTF-8 can be entered
                // * self.index is always on a valid char boundary, as received by char_indices
                // Why use unchecked: We know it is safe (see above), and using the checked fn
                // turns this from O(1) to O(n), which is bad in a loop (which this will be)
                from_utf8_unchecked(&self.val.as_bytes()[self.index.get()..])
            }
            .char_indices();
            indices.next().map(|(_, c)| {
                self.index.set(
                    indices
                        .next()
                        .map_or_else(|| len, |a| self.index.get() + a.0),
                );
                c.into()
            })
        } else {
            Option::None
        }
    }
}

impl AsciiIter {
    fn new(val: AsciiVar) -> AsciiIter {
        AsciiIter {
            val,
            index: Cell::new(0),
        }
    }
}

impl StrIter for AsciiIter {
    fn next_fn(&self) -> Option<Variable> {
        self.val
            .get_ascii(self.index.replace(self.index.get() + 1))
            .map(AsciiChar::as_char)
            .map(Into::into)
    }
}

impl<T> CustomVar for T
where
    T: StrIter,
{
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        if let Name::Attribute(n) = name {
            if &*n == "next" {
                Variable::Method(StdMethod::new_native(self, Self::next_func))
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        unimplemented!()
    }
}

impl<T> NativeIterator for T
where
    T: StrIter + CustomVar,
{
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.next_fn())
    }
}
