use crate::custom_types::list::List;
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

pub fn builtin_of(index: usize) -> Variable {
    return match index {
        0 => print(),
        1 => todo!("Callable"),
        2 => Type::Bigint().into(),
        3 => Type::String().into(),
        4 => Type::Bool().into(),
        5 => todo!("range"),
        6 => Type::Type().into(),
        7 => todo!("iter"),
        8 => todo!("repr"),
        9 => todo!("input"),
        10 => List::list_type().into(),
        11 => todo!("set"),
        12 => todo!("char"),
        13 => todo!("open"),
        _ => unimplemented!(),
    };
}

pub fn default_methods(name: Name) -> StdVarMethod {
    if let Name::Operator(o) = name {
        let result = match o {
            Operator::Repr => default_repr,
            Operator::Str => default_str,
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
