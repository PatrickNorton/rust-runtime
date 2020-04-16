use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_variable::{StdVarMethod, StdVariable};
use crate::variable::{Function, Name, Variable};

fn print() -> Variable {
    Variable::Function(Function::Native(print_impl))
}

fn print_impl(args: Vec<Variable>, runtime: &mut Runtime) {
    for arg in args {
        println!("{}", arg.str(runtime));
    }
}

pub fn builtin_of(index: usize) -> Variable {
    return match index {
        0 => print(),
        _ => todo!(),
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

fn default_repr(this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) {
    debug_assert!(args.is_empty());
    let result = format!("<{}: {}>", this.get_type().to_string(), this.var_ptr());
    runtime.push(Variable::String(result));
}

fn default_str(this: &StdVariable, args: Vec<Variable>, runtime: &mut Runtime) {
    debug_assert!(args.is_empty());
    runtime.call_op(Variable::Standard(this.clone()), Operator::Repr, args);
}
