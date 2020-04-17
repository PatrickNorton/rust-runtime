use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::Variable;
use num::{BigInt, ToPrimitive};
use std::str::FromStr;

pub fn get_operator(this: &StringVar, o: Operator) -> Variable {
    let func: fn(&StringVar, Vec<Variable>, &mut Runtime) = match o {
        Operator::Add => add,
        Operator::Multiply => multiply,
        Operator::Bool => bool,
        Operator::Int => int,
        Operator::Str => str,
        _ => unimplemented!("Operator::{:?} unimplemented", o),
    };
    Variable::Method(Box::new(StdMethod::new(
        this.clone(),
        InnerMethod::Native(func),
    )))
}

fn add(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) {
    let mut result: String = this.parse().unwrap();
    for arg in args {
        result += arg.str(runtime).as_ref();
    }
    runtime.push(Variable::String(result.into()));
}

fn multiply(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) {
    let mut result: String = this.parse().unwrap();
    for arg in args {
        result = result.repeat(
            arg.int(runtime)
                .to_usize()
                .expect("Too many string repetitions"),
        );
    }
    runtime.push(Variable::String(result.into()));
}

fn bool(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bool(this.is_empty()));
}

fn int(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bigint(
        BigInt::from_str(this).expect("Cannot get int value"),
    ));
}

fn str(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) {
    debug_assert!(args.is_empty());
    runtime.push(Variable::String(this.clone()));
}
