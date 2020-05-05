use crate::custom_types::list::List;
use crate::custom_types::range::Range;
use crate::custom_types::set::Set;
use crate::function::Function;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::{StdVarMethod, StdVariable};
use crate::variable::{FnResult, Name, Variable};

fn print() -> Variable {
    Variable::Function(Function::Native(print_impl))
}

fn print_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        println!("{}", arg.str(runtime)?);
    }
    FnResult::Ok(())
}

fn input() -> Variable {
    Variable::Function(Function::Native(input_impl))
}

fn input_impl(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    print!("{}", args[0].str(runtime)?);
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            runtime.push(Variable::String(input.into()));
        }
        Err(_) => panic!("Could not read from stdin"),
    }
    FnResult::Ok(())
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
        13 => todo!("open"),
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
            _ => unimplemented!("name {:?} not found", name),
        };
        StdVarMethod::Native(result)
    } else {
        panic!("name {:?} not found", name)
    }
}

fn default_repr(this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = format!("<{}: {}>", this.get_type().to_string(), this.var_ptr());
    runtime.push(Variable::String(result.into()));
    FnResult::Ok(())
}

fn default_str(this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.call_op(Variable::Standard(this.clone()), Operator::Repr, args)?;
    FnResult::Ok(())
}

fn default_bool(_this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn default_eq(this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let this_var = Variable::Standard(this.clone());
    for arg in args {
        if this_var != arg {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    return FnResult::Ok(());
}
