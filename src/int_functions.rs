use std::boxed::Box;
use std::vec::Vec;

use crate::operator::Operator;
use crate::variable::Variable;
use num_bigint::BigInt;
use crate::runtime::Runtime;
use crate::method::{StdMethod, InnerMethod};
use bigdecimal::BigDecimal;

pub fn get_operator(this: &BigInt, o: Operator) -> Variable {
    let func: fn(&BigInt, &Vec<Variable>, &mut Runtime) = match o {
        Operator::Add => add,
        Operator::Subtract => sub,
        Operator::Multiply => mul,
        Operator::FloorDiv => floor_div,
        Operator::Divide => div,
        _ => unimplemented!(),
    };
    Variable::Method(Box::new(StdMethod::new(this.clone(), InnerMethod::Native(func))))
}

fn add(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut sum = this.clone();
    for arg in args {
        sum += arg.clone().int(runtime)
    }
    runtime.push(Variable::Bigint(sum))
}

fn sub(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut diff = this.clone();
    for arg in args {
        diff -= arg.clone().int(runtime)
    }
    runtime.push(Variable::Bigint(diff))
}

fn mul(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut prod = this.clone();
    for arg in args {
        prod *= arg.clone().int(runtime)
    }
    runtime.push(Variable::Bigint(prod))
}

fn floor_div(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut ratio = this.clone();
    for arg in args {
        ratio *= arg.clone().int(runtime)
    }
    runtime.push(Variable::Bigint(ratio))
}

fn div(this: &BigInt, args: &Vec<Variable>, runtime: &mut Runtime) {
    let mut ratio = BigDecimal::new(this.clone(), 0);
    for arg in args {
        ratio *= arg.clone().int(runtime)
    }
    runtime.push(Variable::Decimal(ratio))
}
