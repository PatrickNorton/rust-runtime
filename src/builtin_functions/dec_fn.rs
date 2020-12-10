use crate::method::{InnerMethod, NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::rational_var::RationalVar;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> NativeMethod<RationalVar> {
    match o {
        Operator::Add => add,
        Operator::Subtract => sub,
        Operator::USubtract => u_minus,
        Operator::Multiply => mul,
        Operator::FloorDiv => floor_div,
        Operator::Divide => div,
        Operator::Equals => eq,
        Operator::LessThan => less_than,
        Operator::GreaterThan => greater_than,
        Operator::LessEqual => less_equal,
        Operator::GreaterEqual => greater_equal,
        Operator::Str => to_str,
        Operator::Int => to_int,
        x => unimplemented!("dec.{}", x.name()),
    }
}

pub fn get_operator(this: RationalVar, o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

fn add(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum: RationalVar = args.into_iter().map(RationalVar::from).sum();
    sum += this.clone();
    runtime.return_1(sum.into())
}

fn sub(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let diff = args
        .into_iter()
        .map(RationalVar::from)
        .fold(this.clone(), |x, y| x - y);
    runtime.return_1(Variable::Decimal(diff))
}

fn u_minus(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Decimal(-this.clone()))
}

fn mul(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut prod: RationalVar = args.into_iter().map(RationalVar::from).product();
    prod += this.clone();
    runtime.return_1(Variable::Decimal(prod))
}

fn floor_div(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = this.clone().to_integer();
    for arg in args {
        ratio /= RationalVar::from(arg).to_integer()
    }
    runtime.return_1(Variable::Bigint(ratio.into()))
}

fn div(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = this.clone();
    for arg in args {
        ratio /= RationalVar::from(arg)
    }
    runtime.return_1(Variable::Decimal(ratio))
}

fn eq(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let eq = args.into_iter().all(|x| *this == RationalVar::from(x));
    runtime.return_1(Variable::Bool(eq))
}

fn less_than(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let lt = args.into_iter().all(|x| *this < RationalVar::from(x));
    runtime.return_1(Variable::Bool(lt))
}

fn greater_than(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let gt = args.into_iter().all(|x| *this > RationalVar::from(x));
    runtime.return_1(Variable::Bool(gt))
}

fn less_equal(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let le = args.into_iter().all(|x| *this <= RationalVar::from(x));
    runtime.return_1(Variable::Bool(le))
}

fn greater_equal(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let ge = args.into_iter().all(|x| *this >= RationalVar::from(x));
    runtime.return_1(Variable::Bool(ge))
}

fn to_str(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String(format!("{}", **this).into()))
}

fn to_int(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Bigint(this.to_integer().into()))
}
