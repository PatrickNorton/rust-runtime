use crate::builtin_functions::int_fn;
use crate::int_var::IntVar;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::{FnResult, FromBool, Variable};

pub fn op_fn(o: Operator) -> Option<NativeMethod<bool>> {
    Option::Some(match o {
        Operator::Equals => eq,
        Operator::LessThan => less_than,
        Operator::GreaterThan => greater_than,
        Operator::LessEqual => less_equal,
        Operator::GreaterEqual => greater_equal,
        Operator::BitwiseAnd => bitwise_and,
        Operator::BitwiseOr => bitwise_or,
        Operator::BitwiseNot => bitwise_not,
        Operator::BitwiseXor => bitwise_xor,
        Operator::Str => str,
        Operator::Repr => str,
        _ => return Option::None,
    })
}

pub fn get_operator(this: bool, o: Operator) -> Variable {
    match op_fn(o) {
        Option::Some(func) => Variable::Method(StdMethod::new_native(this, func)),
        Option::None => int_fn::get_operator(IntVar::from_bool(this), o),
    }
}

fn eq(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if bool::from(arg) != *this {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn less_than(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this >= bool::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn greater_than(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this <= bool::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn less_equal(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this > bool::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn greater_equal(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this < bool::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn bitwise_and(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum &= &bool::from(arg)
    }
    runtime.push(Variable::Bool(sum));
    FnResult::Ok(())
}

fn bitwise_or(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum |= &bool::from(arg)
    }
    runtime.push(Variable::Bool(sum));
    FnResult::Ok(())
}

fn bitwise_not(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bool(!*this));
    FnResult::Ok(())
}

fn bitwise_xor(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum ^= &bool::from(arg)
    }
    runtime.push(Variable::Bool(sum));
    FnResult::Ok(())
}

fn str(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::String(
        if *this { "true" } else { "false" }.into(),
    ));
    FnResult::Ok(())
}
