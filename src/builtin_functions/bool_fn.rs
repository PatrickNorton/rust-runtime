use crate::builtin_functions::int_fn;
use crate::int_var::IntVar;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::var_impls::FromBool;
use crate::variable::{FnResult, Variable};

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
        Option::Some(func) => StdMethod::new_native(this, func).into(),
        Option::None => int_fn::get_operator(IntVar::from_bool(this), o),
    }
}

fn eq(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if bool::from(arg) != this {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn less_than(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this >= bool::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn greater_than(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this <= bool::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn less_equal(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this & !bool::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn greater_equal(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if !this & bool::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn bitwise_and(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum &= bool::from(arg)
    }
    runtime.return_1(sum.into())
}

fn bitwise_or(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum |= bool::from(arg)
    }
    runtime.return_1(sum.into())
}

fn bitwise_not(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1((!this).into())
}

fn bitwise_xor(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum ^= bool::from(arg)
    }
    runtime.return_1(sum.into())
}

fn str(this: bool, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(StringVar::from(if this { "true" } else { "false" }).into());
    runtime.return_0()
}
