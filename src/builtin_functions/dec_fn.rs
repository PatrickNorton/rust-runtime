use crate::int_var::IntVar;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::rational_var::RationalVar;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
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
    StdMethod::new_native(this, func).into()
}

fn add(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum: RationalVar = args.into_iter().map(RationalVar::from).sum();
    sum += this;
    runtime.return_1(sum.into())
}

fn sub(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let diff = args
        .into_iter()
        .map(RationalVar::from)
        .fold(this, |x, y| x - y);
    runtime.return_1(diff.into())
}

fn u_minus(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1((-this).into())
}

fn mul(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut prod: RationalVar = args.into_iter().map(RationalVar::from).product();
    prod += this;
    runtime.return_1(prod.into())
}

fn floor_div(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = this.to_integer();
    for arg in args {
        ratio /= RationalVar::from(arg).to_integer()
    }
    runtime.return_1(ratio.into())
}

fn div(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = this;
    for arg in args {
        ratio /= RationalVar::from(arg)
    }
    runtime.return_1(ratio.into())
}

fn eq(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let eq = args.into_iter().all(|x| this == RationalVar::from(x));
    runtime.return_1(eq.into())
}

fn less_than(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let lt = args.into_iter().all(|x| this < RationalVar::from(x));
    runtime.return_1(lt.into())
}

fn greater_than(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let gt = args.into_iter().all(|x| this > RationalVar::from(x));
    runtime.return_1(gt.into())
}

fn less_equal(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let le = args.into_iter().all(|x| this <= RationalVar::from(x));
    runtime.return_1(le.into())
}

fn greater_equal(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let ge = args.into_iter().all(|x| this >= RationalVar::from(x));
    runtime.return_1(ge.into())
}

fn to_str(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(StringVar::from(format!("{}", *this)).into())
}

fn to_int(this: RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_integer().into())
}
