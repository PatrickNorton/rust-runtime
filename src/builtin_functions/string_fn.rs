use crate::custom_types::bytes::LangBytes;
use crate::custom_types::exceptions::{arithmetic_error, index_error, value_error};
use crate::custom_types::list::List;
use crate::custom_types::range::Range;
use crate::custom_var::CustomVar;
use crate::function::Function;
use crate::int_var::IntVar;
use crate::looping::{IterResult, NativeIterator};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::{AsciiVar, MaybeAscii, StrVar, StringVar};
use crate::variable::{FnResult, Variable};
use crate::{first, first_two};
use ascii::{AsAsciiStr, AsciiChar, AsciiStr, AsciiString};
use downcast_rs::Downcast;
use num::{BigInt, Num, One, Signed, ToPrimitive};
use std::any::Any;
use std::cell::Cell;
use std::fmt::Debug;
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
        Operator::GetSlice => slice,
        Operator::Iter => str_iter,
        Operator::Reversed => reversed,
        _ => unimplemented!("str.{}", o.name()),
    }
}

pub fn get_operator(this: StringVar, o: Operator) -> Variable {
    StdMethod::new_native(this, op_fn(o)).into()
}

pub fn get_attr(this: StringVar, s: &str) -> Variable {
    let func = match s {
        "length" => return IntVar::from(this.chars().count()).into(),
        "upper" => upper,
        "lower" => lower,
        "join" => join,
        "joinAll" => join_all,
        "startsWith" => starts_with,
        "endsWith" => ends_with,
        "split" => split,
        "splitlines" => split_lines,
        "indexOf" => index_of,
        "lastIndexOf" => last_index_of,
        "chars" => return chars(&this),
        "encode" => encode,
        "intBase" => int_base,
        "asInt" => as_int,
        x => unimplemented!("str.{}", x),
    };
    StdMethod::new_native(this, func).into()
}

pub fn static_attr(s: &str) -> Variable {
    let func = match s {
        "fromChars" => from_chars,
        x => unimplemented!("str.{}", x),
    };
    Function::Native(func).into()
}

fn add(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let result = args.into_iter().fold(this.to_string(), |acc, arg| {
        acc + StringVar::from(arg).as_ref()
    });
    runtime.return_1(StringVar::from(result).into())
}

fn multiply(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if this.is_empty() {
        return runtime.return_1(this.into());
    }
    if args.len() == 1 && args[0].as_int().map_or_else(|| false, One::is_one) {
        return runtime.return_1(this.into());
    }
    match this.as_maybe_ascii() {
        MaybeAscii::Standard(s) => mul_str(s.to_owned(), args, runtime),
        MaybeAscii::Ascii(a) => mul_ascii(a.to_owned(), args, runtime),
    }
}

fn mul_ascii(mut result: AsciiString, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        let big_val = IntVar::from(arg);
        match big_val.to_usize() {
            Option::Some(val) => match val.checked_mul(result.len()) {
                Option::Some(_) => result = result.as_slice().repeat(val).into(),
                Option::None => {
                    return runtime.throw_quick(arithmetic_error(), overflow_exc(val, result.len()))
                }
            },
            Option::None => return runtime.throw_quick(arithmetic_error(), mul_exc(big_val)),
        }
    }
    runtime.return_1(StringVar::from(result).into())
}

fn mul_str(mut result: String, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
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
    runtime.return_1(StringVar::from(result).into())
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

fn overflow_exc(val: usize, len: usize) -> String {
    format!(
        "Too many string repetitions: maximum string length is {}, \
        but repetition would produce str of length {}",
        usize::MAX,
        BigInt::from(val) * len
    )
}

fn bool(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.is_empty().into())
}

fn int(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    match IntVar::from_str(&*this) {
        Ok(val) => runtime.return_1(val.into()),
        Err(_) => runtime.throw_quick(
            value_error(),
            format!(
                "Invalid input for int(str): {} is not a valid base-10 integer",
                this.repr()
            ),
        ),
    }
}

fn str(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.into())
}

fn repr(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.repr().into())
}

fn index(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let big_index = IntVar::from(first(args));
    match this.as_maybe_ascii() {
        MaybeAscii::Standard(s) => index_non_ascii(s, big_index, runtime),
        MaybeAscii::Ascii(a) => index_ascii(a, big_index, runtime),
    }
}

fn index_non_ascii(s: &str, big_index: IntVar, runtime: &mut Runtime) -> FnResult {
    let value = if big_index.is_negative() {
        // Instead of getting the character length and then indexing from the start, index from
        // the back instead, which will only result in iterating through the string once
        to_abs_usize(&big_index).and_then(|b| s.chars().nth(b - 1))
    } else {
        big_index.to_usize().and_then(|b| s.chars().nth(b))
    };
    match value {
        Option::None => {
            runtime.throw_quick(index_error(), bounds_msg(&big_index, s.chars().count()))
        }
        Option::Some(chr) => runtime.return_1(chr.into()),
    }
}

fn index_ascii(a: &AsciiStr, big_index: IntVar, runtime: &mut Runtime) -> FnResult {
    let proper_index = if big_index.is_negative() {
        (&big_index + &a.len().into()).to_usize()
    } else {
        big_index.to_usize()
    };
    match proper_index.and_then(|index| a.get_ascii(index)) {
        Option::Some(chr) => runtime.return_1(chr.into()),
        Option::None => runtime.throw_quick(index_error(), bounds_msg(&big_index, a.len())),
    }
}

// Prevents unnecessary clone of `i`
fn to_abs_usize(i: &IntVar) -> Option<usize> {
    match i {
        IntVar::Small(s) => Option::Some(s.abs() as usize), // unsigned_abs() is gated behind #74913
        IntVar::Big(b) => b.magnitude().to_usize(),
    }
}

fn slice(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let range = Range::from_slice(this.char_len(), runtime, first(args))?;
    if range.get_step().is_one() {
        let start = range.get_start();
        let stop = range.get_stop();
        let len = this.char_len();
        match to_pair(start, stop, len) {
            Result::Ok((x, y)) => runtime.return_1(slice_single(&this, x, y).into()),
            Result::Err(i) => {
                let msg = bounds_msg(i, len);
                runtime.throw_quick(index_error(), msg)
            }
        }
    } else {
        slice_normal(this.as_maybe_ascii(), &range, runtime)
    }
}

fn to_pair<'a>(
    start: &'a IntVar,
    stop: &'a IntVar,
    len: usize,
) -> Result<(usize, usize), &'a IntVar> {
    match (
        start.to_usize().filter(|x| *x <= len),
        stop.to_usize().filter(|x| *x <= len),
    ) {
        (Option::Some(x), Option::Some(y)) => Result::Ok((x, y)),
        (Option::Some(_), Option::None) => Result::Err(stop),
        (Option::None, Option::Some(_)) => Result::Err(start),
        (Option::None, Option::None) => Result::Err(start),
    }
}

fn slice_single(this: &StringVar, start: usize, stop: usize) -> StringVar {
    match this {
        // This can be made into a literal, but it can't be done safely and thus not worth it here
        StringVar::Literal(l) => l
            .chars()
            .skip(start)
            .take(stop - start)
            .collect::<String>()
            .into(),
        StringVar::AsciiLiteral(a) => a[start..stop].into(),
        StringVar::Other(o) => o
            .chars()
            .skip(start)
            .take(stop - start)
            .collect::<String>()
            .into(),
        StringVar::Ascii(a) => a[start..stop].to_owned().into(),
    }
}

fn slice_normal(this: MaybeAscii, range: &Range, runtime: &mut Runtime) -> FnResult {
    match this {
        MaybeAscii::Ascii(s) => {
            let mut result = AsciiString::new();
            for i in range.values() {
                let index = match i.to_usize() {
                    Option::Some(val) => val,
                    Option::None => {
                        let msg = bounds_msg(&i, s.len());
                        return runtime.throw_quick(index_error(), msg);
                    }
                };
                match s.get_ascii(index) {
                    Option::None => {
                        let msg = bounds_msg(&i, s.len());
                        return runtime.throw_quick(index_error(), msg);
                    }
                    Option::Some(value) => result.push(value),
                }
            }
            runtime.return_1(StringVar::from(result).into())
        }
        MaybeAscii::Standard(s) => {
            let mut result = String::new();
            for i in range.values() {
                let index = match i.to_usize() {
                    Option::Some(val) => val,
                    Option::None => {
                        let msg = bounds_msg(&i, s.chars().count());
                        return runtime.throw_quick(index_error(), msg);
                    }
                };
                match s.chars().nth(index) {
                    Option::None => {
                        let msg = bounds_msg(&i, s.chars().count());
                        return runtime.throw_quick(index_error(), msg);
                    }
                    Option::Some(value) => result.push(value),
                }
            }
            runtime.return_1(StringVar::from(result).into())
        }
    }
}

fn bounds_msg(big_index: &IntVar, char_len: usize) -> StringVar {
    format!(
        "Index {} out of bounds for str of length {}",
        big_index, char_len
    )
    .into()
}

pub fn iter(this: StringVar) -> Rc<dyn NativeIterator> {
    match this.split_ascii() {
        Result::Ok(a) => Rc::new(AsciiIter::new(a)),
        Result::Err(s) => Rc::new(StringIter::new(s)),
    }
}

fn str_iter(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(match this.split_ascii() {
        Result::Ok(a) => Rc::new(AsciiIter::new(a)).into(),
        Result::Err(s) => Rc::new(StringIter::new(s)).into(),
    })
}

fn reversed(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.chars().rev().collect::<String>().into())
}

fn upper(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_uppercase().into())
}

fn lower(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_lowercase().into())
}

fn join(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let iter = first(args).iter(runtime)?;
    if let Option::Some(val) = iter.next(runtime)?.take_first() {
        let mut result = val.str(runtime)?.as_owned();
        while let Option::Some(val) = iter.next(runtime)?.take_first() {
            result += &this;
            result += &val.str(runtime)?;
        }
        runtime.return_1(StringVar::from(result).into())
    } else {
        runtime.return_1(StringVar::default().into())
    }
}

fn join_all(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut result = String::with_capacity(this.char_len() * args.len());
    let len = args.len();
    for (i, val) in args.into_iter().enumerate() {
        result += val.str(runtime)?.as_str();
        if i < len - 1 {
            result += &*this;
        }
    }
    runtime.return_1(result.into())
}

fn starts_with(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 2);
    let (a, b) = first_two(args);
    let val = StringVar::from(a);
    let index = IntVar::from(b);
    match index.to_usize().filter(|x| *x < this.char_len()) {
        Option::Some(usize_index) => {
            if usize_index == 0 {
                runtime.return_1(this.starts_with(val.as_str()).into())
            } else {
                let mut chars = this.chars();
                chars.nth(usize_index - 1);
                runtime.return_1(chars.as_str().starts_with(val.as_str()).into())
            }
        }
        Option::None => runtime.throw_quick(index_error(), ""),
    }
}

fn ends_with(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let val = StringVar::from(first(args));
    runtime.return_1(this.ends_with(&*val).into())
}

fn split(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 2);
    let (a, opt_count) = first_two(args);
    let pat = StringVar::from(a);
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

fn split_lines(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = List::from_values(Type::String, this.owned_lines().map(From::from).collect());
    runtime.return_1(result.into())
}

fn chars(this: &str) -> Variable {
    List::from_values(Type::Char, this.chars().map(Variable::from).collect()).into()
}

fn from_chars(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let mut result = String::new();
    let chars = first(args).iter(runtime)?;
    while let Option::Some(val) = chars.next(runtime)?.take_first() {
        result.push(val.into());
    }
    runtime.return_1(result.into())
}

fn index_of(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let chr: char = first(args).into();
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

fn last_index_of(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let chr: char = first(args).into();
    let index = match this.as_maybe_ascii() {
        MaybeAscii::Standard(s) => {
            // Needed because str.chars() is not ExactSize
            let mut iter = s.chars().rev().enumerate();
            let index = iter.find(|(_, c)| *c == chr).map(|(i, _)| i);
            let length = iter.last().map(|(i, _)| i);
            // If length is None, then index was the first char, so the result is 0
            // If index is None, then the char was not found
            index.map(|x| length.map(|y| y - x - 1).unwrap_or(0))
        }
        MaybeAscii::Ascii(a) => a
            .chars()
            .enumerate()
            .rev()
            .find(|(_, c)| *c == chr)
            .map(|(i, _)| i),
    };
    runtime.return_1(index.map(IntVar::from).map(Variable::from).into())
}

fn encode(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    // #![feature(array_value_iter)] will make this so much easier...
    let byte_val = match first(args).str(runtime)?.to_lowercase().as_str() {
        "ascii" => match AsciiStr::from_ascii(this.as_str()) {
            Result::Ok(s) => s.as_bytes().to_vec(),
            Result::Err(err) => {
                return runtime.throw_quick(
                    value_error(),
                    format!(
                        "Cannot convert to ascii: byte at position {} (value {}) is not in the range [0:128]", 
                        err.valid_up_to(), this.as_str().as_bytes()[err.valid_up_to()]
                    )
                )
            }
        }
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
                format!("{} is not a valid encoding", x),
            )
        }
    };
    runtime.return_1(Rc::new(LangBytes::new(byte_val.to_vec())).into())
}

fn int_base(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let base: IntVar = first(args).into();
    match base.to_u32().filter(|x| (2..=32).contains(x)) {
        Option::Some(x) => runtime.return_1(
            IntVar::from_str_radix(&this, x)
                .ok()
                .map(Variable::from)
                .into(),
        ),
        Option::None => runtime.throw_quick(
            value_error(),
            format!(
                "str.intBase requires a radix between 2 and 36, not {}",
                base
            ),
        ),
    }
}

fn as_int(this: StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(IntVar::from_str(&*this).ok().map(Variable::from).into())
}

pub trait StrIter: Debug + Any + Downcast {
    fn next_fn(&self) -> Option<Variable>;

    fn next_func(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
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
            if n == "next" {
                StdMethod::new_native(self, Self::next_func).into()
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

    fn get_type(&self) -> Type {
        unimplemented!()
    }
}

impl<T> NativeIterator for T
where
    T: StrIter + CustomVar,
{
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.next_fn().into())
    }
}
