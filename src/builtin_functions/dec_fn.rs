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
        _ => unimplemented!(),
    }
}

pub fn get_operator(this: RationalVar, o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

fn add(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum += RationalVar::from(arg)
    }
    runtime.push(Variable::Decimal(sum));
    FnResult::Ok(())
}

fn sub(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut diff = this.clone();
    for arg in args {
        diff -= RationalVar::from(arg)
    }
    runtime.push(Variable::Decimal(diff));
    FnResult::Ok(())
}

fn u_minus(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Decimal(-this.clone()));
    FnResult::Ok(())
}

fn mul(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut prod = this.clone();
    for arg in args {
        prod -= RationalVar::from(arg)
    }
    runtime.push(Variable::Decimal(prod));
    FnResult::Ok(())
}

fn floor_div(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = this.clone().to_integer();
    for arg in args {
        ratio /= RationalVar::from(arg).to_integer()
    }
    runtime.push(Variable::Bigint(ratio.into()));
    FnResult::Ok(())
}

fn div(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = this.clone();
    for arg in args {
        ratio /= RationalVar::from(arg)
    }
    runtime.push(Variable::Decimal(ratio));
    FnResult::Ok(())
}

fn eq(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if RationalVar::from(arg) != *this {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn less_than(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this >= RationalVar::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn greater_than(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this <= RationalVar::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn less_equal(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this > RationalVar::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn greater_equal(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this < RationalVar::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn to_str(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::String(format!("{}", **this).into()));
    FnResult::Ok(())
}

fn to_int(this: &RationalVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bigint(this.to_integer().into()));
    FnResult::Ok(())
}
