use std::boxed::Box;
use std::vec::Vec;

use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::Variable;
use num::bigint::BigInt;
use num::BigRational;

pub fn get_operator(this: &BigInt, o: Operator) -> Variable {
    let func: fn(&BigInt, &Vec<Variable>, &mut Runtime) = match o {
        Operator::Add => add,
        Operator::Subtract => sub,
        Operator::Multiply => mul,
        Operator::FloorDiv => floor_div,
        Operator::Divide => div,
        Operator::LessThan => less_than,
        _ => unimplemented!(),
    };
    Variable::Method(Box::new(StdMethod::new(
        this.clone(),
        InnerMethod::Native(func),
    )))
}

fn add(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut sum = this.clone();
    for arg in args {
        sum += arg.int(runtime)
    }
    runtime.push(Variable::Bigint(sum))
}

fn sub(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut diff = this.clone();
    for arg in args {
        diff -= arg.int(runtime)
    }
    runtime.push(Variable::Bigint(diff))
}

fn mul(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut prod = this.clone();
    for arg in args {
        prod *= arg.int(runtime)
    }
    runtime.push(Variable::Bigint(prod))
}

fn floor_div(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut ratio = this.clone();
    for arg in args {
        ratio *= arg.int(runtime)
    }
    runtime.push(Variable::Bigint(ratio))
}

fn div(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut ratio = BigRational::from_integer(this.clone());
    for arg in args {
        ratio *= BigRational::from_integer(arg.int(runtime))
    }
    runtime.push(Variable::Decimal(ratio))
}

fn less_than(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    for arg in args {
        if arg.int(runtime) > *this {
            runtime.push(Variable::Bool(true));
        }
    }
    runtime.push(Variable::Bool(false));
}
