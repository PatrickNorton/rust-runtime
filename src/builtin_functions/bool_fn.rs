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
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn less_than(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this >= bool::from(arg) {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn greater_than(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this <= bool::from(arg) {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn less_equal(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this > bool::from(arg) {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn greater_equal(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this < bool::from(arg) {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn bitwise_and(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum &= &bool::from(arg)
    }
    runtime.return_1(Variable::Bool(sum))
}

fn bitwise_or(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum |= &bool::from(arg)
    }
    runtime.return_1(Variable::Bool(sum))
}

fn bitwise_not(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Bool(!*this))
}

fn bitwise_xor(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum ^= &bool::from(arg)
    }
    runtime.return_1(Variable::Bool(sum))
}

fn str(this: &bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::String(
        if *this { "true" } else { "false" }.into(),
    ));
    FnResult::Ok(())
}
