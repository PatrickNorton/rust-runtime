use crate::custom_types::array::Array;
use crate::custom_types::bytes::LangBytes;
use crate::custom_types::dict::Dict;
use crate::custom_types::enumerate::Enumerate;
use crate::custom_types::exceptions::{io_error, not_implemented, value_error};
use crate::custom_types::file::FileObj;
use crate::custom_types::interfaces::{Callable, Throwable};
use crate::custom_types::list::List;
use crate::custom_types::range::Range;
use crate::custom_types::set::Set;
use crate::custom_types::slice::Slice;
use crate::function::Function;
use crate::int_var::IntVar;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::{StdVarMethod, StdVariable};
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::mem::take;

fn print() -> Variable {
    Function::Native(print_impl).into()
}

fn print_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        println!("{}", arg.str(runtime)?);
    }
    runtime.return_0()
}

fn input() -> Variable {
    Function::Native(input_impl).into()
}

fn input_impl(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    print!("{}", take(&mut args[0]).str(runtime)?);
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => runtime.push(StringVar::from(input).into()),
        Err(_) => runtime.throw_quick(io_error(), "Could not read from stdin".into())?,
    }
    runtime.return_0()
}

fn repr() -> Variable {
    Function::Native(repr_impl).into()
}

fn repr_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    runtime.call_op(args[0].clone(), Operator::Repr, Vec::new())
}

fn iter() -> Variable {
    Function::Native(iter_impl).into()
}

fn iter_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    runtime.call_op(args[0].clone(), Operator::Iter, Vec::new())
}

fn reversed() -> Variable {
    Function::Native(reversed_impl).into()
}

fn reversed_impl(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    runtime.call_op(take(&mut args[0]), Operator::Reversed, Vec::new())
}

fn id() -> Variable {
    Function::Native(id_impl).into()
}

fn id_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    runtime.return_1(IntVar::from(args[0].id()).into())
}

fn enumerate() -> Variable {
    Function::Native(enumerate_impl).into()
}

fn enumerate_impl(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let iterable = take(&mut args[0]).iter(runtime)?;
    runtime.return_1(Enumerate::new(iterable).into())
}

fn hash() -> Variable {
    Function::Native(hash_impl).into()
}

fn hash_impl(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let hash = take(&mut args[0]).hash(runtime)?;
    runtime.return_1(IntVar::from(hash).into())
}

pub fn builtin_of(index: usize) -> Variable {
    match index {
        0 => print(),
        1 => Callable::cls().into(),
        2 => Type::Bigint.into(),
        3 => Type::String.into(),
        4 => Type::Bool.into(),
        5 => Range::range_type().into(),
        6 => Type::Type.into(),
        7 => iter(),
        8 => repr(),
        9 => input(),
        10 => List::list_type().into(),
        11 => Set::set_type().into(),
        12 => Type::Char.into(),
        13 => FileObj::open_type().into(),
        14 => reversed(),
        15 => Slice::slice_type().into(),
        16 => id(),
        17 => Array::array_type().into(),
        18 => enumerate(),
        19 => LangBytes::bytes_type().into(),
        20 => Dict::dict_type().into(),
        21 => Type::Object.into(),
        22 => not_implemented().into(),
        23 => Type::Tuple.into(),
        24 => Throwable::cls().into(),
        25 => Type::Null.into(),
        26 => hash(),
        27 => value_error().into(),
        x => unimplemented!("Builtin number {}", x),
    }
}

pub fn default_methods(name: Name) -> Option<StdVarMethod> {
    if let Name::Operator(o) = name {
        let result = match o {
            Operator::Repr => default_repr,
            Operator::Str => default_str,
            Operator::Equals => default_eq,
            Operator::Bool => default_bool,
            Operator::In => default_in,
            _ => return Option::None,
        };
        Option::Some(StdVarMethod::Move(result))
    } else {
        Option::None
    }
}

fn default_repr(this: StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = format!("<{}: 0x{:X}>", this.get_type().to_string(), this.var_ptr());
    runtime.return_1(StringVar::from(result).into())
}

fn default_str(this: StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.call_op(this.clone().into(), Operator::Repr, args)
}

fn default_bool(_this: StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(true.into())
}

fn default_eq(this: StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let this_var: Variable = this.clone().into();
    for arg in args {
        if this_var != arg {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn default_in(this: StdVariable, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let checked_var = take(&mut args[0]);
    let this_iter = this.iter(runtime)?;
    while let Option::Some(val) = this_iter.next(runtime)?.take_first() {
        if checked_var.equals(val, runtime)? {
            return runtime.return_1(true.into());
        }
    }
    runtime.return_1(false.into())
}
