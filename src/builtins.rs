use crate::runtime::Runtime;
use crate::variable::{Function, Variable};

fn print() -> Variable {
    Variable::Function(Function::Native(print_impl))
}

fn print_impl(args: &Vec<Variable>, runtime: &mut Runtime) {
    for arg in args {
        println!("{}", arg.str(runtime));
    }
}

pub fn builtin_of(index: usize) -> Variable {
    println!("{}", index);
    return match index {
        0 => print(),
        _ => todo!(),
    };
}
