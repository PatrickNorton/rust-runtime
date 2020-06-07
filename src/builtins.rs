use crate::custom_types::exceptions::io_error;
use crate::custom_types::file::FileObj;
use crate::custom_types::list::List;
use crate::custom_types::range::Range;
use crate::custom_types::set::Set;
use crate::function::Function;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::{StdVarMethod, StdVariable};
use crate::variable::{FnResult, Variable};
use std::mem::{replace, take};

fn print() -> Variable {
    Variable::Function(Function::Native(print_impl))
}

fn print_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        println!("{}", arg.str(runtime)?);
    }
    runtime.return_0()
}

fn input() -> Variable {
    Variable::Function(Function::Native(input_impl))
}

fn input_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    print!("{}", args[0].str(runtime)?);
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => runtime.push(Variable::String(input.into())),
        Err(_) => runtime.throw_quick(io_error(), "Could not read from stdin".into())?,
    }
    runtime.return_0()
}

fn repr() -> Variable {
    Variable::Function(Function::Native(repr_impl))
}

fn repr_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    runtime.call_op(args[0].clone(), Operator::Repr, Vec::new())
}

fn iter() -> Variable {
    Variable::Function(Function::Native(iter_impl))
}

fn iter_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    runtime.call_op(args[0].clone(), Operator::Iter, Vec::new())
}

fn reversed() -> Variable {
    Variable::Function(Function::Native(reversed_impl))
}

fn reversed_impl(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    runtime.call_op(take(&mut args[0]), Operator::Reversed, Vec::new())
}

pub fn builtin_of(index: usize) -> Variable {
    match index {
        0 => print(),
        1 => todo!("Callable"),
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
        _ => unimplemented!(),
    }
}

pub fn default_methods(name: Name) -> StdVarMethod {
    if let Name::Operator(o) = name {
        let result = match o {
            Operator::Repr => default_repr,
            Operator::Str => default_str,
            Operator::Equals => default_eq,
            Operator::Bool => default_bool,
            Operator::In => default_in,
            _ => unimplemented!("name {:?} not found", name),
        };
        StdVarMethod::Native(result)
    } else {
        panic!("name {:?} not found", name)
    }
}

fn default_repr(this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = format!("<{}: 0x{:X}>", this.get_type().to_string(), this.var_ptr());
    runtime.return_1(Variable::String(result.into()))
}

fn default_str(this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.call_op(Variable::Standard(this.clone()), Operator::Repr, args)
}

fn default_bool(_this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Bool(true))
}

fn default_eq(this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let this_var = Variable::Standard(this.clone());
    for arg in args {
        if this_var != arg {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn default_in(this: &StdVariable, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let checked_var = replace(&mut args[0], Variable::Null());
    let this_iter = this.iter(runtime)?;
    while let Option::Some(val) = this_iter.clone().next(runtime)? {
        if checked_var.equals(val, runtime)? {
            return runtime.return_1(true.into());
        }
    }
    runtime.return_1(false.into())
}
