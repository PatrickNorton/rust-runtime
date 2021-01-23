use crate::custom_types::exceptions::{arithmetic_error, value_error};
use crate::first;
use crate::int_var::IntVar;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::rational_var::RationalVar;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::tuple::LangTuple;
use crate::variable::{FnResult, Variable};
use num::traits::Pow;
use num::{Integer, Signed, ToPrimitive, Zero};
use std::ops::Neg;
use std::vec::Vec;

pub fn op_fn(o: Operator) -> NativeMethod<IntVar> {
    match o {
        Operator::Add => add,
        Operator::Subtract => sub,
        Operator::USubtract => u_minus,
        Operator::Multiply => mul,
        Operator::FloorDiv => floor_div,
        Operator::Divide => div,
        Operator::Power => pow,
        Operator::Modulo => modulo,
        Operator::Equals => eq,
        Operator::LessThan => less_than,
        Operator::GreaterThan => greater_than,
        Operator::LessEqual => less_equal,
        Operator::GreaterEqual => greater_equal,
        Operator::LeftBitshift => left_bs,
        Operator::RightBitshift => right_bs,
        Operator::BitwiseAnd => bitwise_and,
        Operator::BitwiseOr => bitwise_or,
        Operator::BitwiseNot => bitwise_not,
        Operator::BitwiseXor => bitwise_xor,
        Operator::Str => to_str,
        Operator::Int => to_int,
        Operator::Repr => to_str,
        Operator::Bool => to_bool,
        _ => unimplemented!("int.{} unimplemented", o.name()),
    }
}

pub fn get_operator(this: IntVar, o: Operator) -> Variable {
    let func = op_fn(o);
    StdMethod::new_native(this, func).into()
}

pub fn str_fn(s: &str) -> NativeMethod<IntVar> {
    match s {
        "strBase" => str_base,
        "divRem" => div_rem,
        _ => unimplemented!("int.{} unimplemented", s),
    }
}

pub fn get_attribute(this: IntVar, s: &str) -> Variable {
    let func = str_fn(s);
    StdMethod::new_native(this, func).into()
}

fn add(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        return runtime.return_1((this + first(args).into()).into());
    }
    let mut sum = this;
    for arg in args {
        sum += IntVar::from(arg)
    }
    runtime.return_1(sum.into())
}

fn sub(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        return runtime.return_1((this - first(args).into()).into());
    }
    let mut diff = this;
    for arg in args {
        diff -= IntVar::from(arg)
    }
    runtime.return_1(diff.into())
}

fn u_minus(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.neg().into())
}

fn mul(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        return runtime.return_1((this * first(args).into()).into());
    }
    let mut prod = this;
    for arg in args {
        prod *= IntVar::from(arg)
    }
    runtime.return_1(prod.into())
}

fn floor_div(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        let var: IntVar = first(args).into();
        if var.is_zero() {
            return runtime.throw_quick(value_error(), "Cannot divide by 0");
        }
        return runtime.return_1((this / var).into());
    }
    let mut ratio = this;
    for arg in args {
        let var = IntVar::from(arg);
        if var.is_zero() {
            return runtime.throw_quick(value_error(), "Cannot divide by 0");
        }
        ratio /= var;
    }
    runtime.return_1(ratio.into())
}

fn div(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = RationalVar::from_integer(this.into());
    for arg in args {
        let var = IntVar::from(arg);
        if var.is_zero() {
            return runtime.throw_quick(value_error(), "Cannot divide by 0");
        }
        ratio /= RationalVar::from_integer(var.into())
    }
    runtime.return_1(ratio.into())
}

fn pow(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let arg_int = IntVar::from(first(args));
    let result = this.pow(arg_int);
    runtime.return_1(result.into())
}

fn modulo(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let arg_int = IntVar::from(first(args));
    if arg_int.is_zero() {
        runtime.throw_quick(value_error(), "Cannot modulo by 0")
    } else {
        runtime.return_1((this % arg_int).into())
    }
}

fn eq(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if IntVar::from(arg) != this {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn less_than(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this >= IntVar::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn greater_than(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this <= IntVar::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn less_equal(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this > IntVar::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn greater_equal(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if this < IntVar::from(arg) {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn left_bs(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let big_value = IntVar::from(first(args));
    let result = match big_value.to_usize() {
        Option::None => {
            let msg = if big_value.is_negative() {
                format!("Cannot shift by {} (min shift value is 0)", big_value)
            } else {
                format!(
                    "Cannot shift by {} (max shift value is {})",
                    big_value,
                    usize::MAX
                )
            };
            return runtime.throw_quick(arithmetic_error(), msg);
        }
        Option::Some(b) => this << b,
    };
    runtime.return_1(result.into())
}

fn right_bs(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let big_value = IntVar::from(first(args));
    let result = match big_value.to_usize() {
        Option::None => {
            let msg = if big_value.is_negative() {
                format!("Cannot shift by {} (min shift value is 0)", big_value)
            } else {
                format!(
                    "Cannot shift by {} (max shift value is {})",
                    big_value,
                    usize::MAX
                )
            };
            return runtime.throw_quick(arithmetic_error(), msg);
        }
        Option::Some(b) => this >> b,
    };
    runtime.return_1(result.into())
}

fn bitwise_and(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum &= IntVar::from(arg)
    }
    runtime.return_1(sum.into())
}

fn bitwise_or(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum |= IntVar::from(arg)
    }
    runtime.return_1(sum.into())
}

fn bitwise_not(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1((!this).into())
}

fn bitwise_xor(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this;
    for arg in args {
        sum ^= IntVar::from(arg)
    }
    runtime.return_1(sum.into())
}

fn to_str(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    match args.len() {
        0 => runtime.return_1(StringVar::from(this.to_string()).into()),
        1 => {
            let variable = IntVar::from(first(args));
            if variable >= 2.into() && variable <= 36.into() {
                runtime
                    .return_1(StringVar::from(this.to_str_radix(variable.to_u32().unwrap())).into())
            } else {
                runtime.throw_quick(
                    value_error(),
                    format!(
                        "Invalid radix for int.to_str: Expected in [2:37], got {}",
                        variable
                    ),
                )
            }
        }
        x => panic!(
            "Expected 1 or 2 arguments for int.operator str, got {}\n{}",
            x,
            runtime.frame_strings()
        ),
    }
}

fn to_int(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.into())
}

fn to_bool(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1((!this.is_zero()).into())
}

fn str_base(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let value: IntVar = first(args).into();
    match value.to_u32().filter(|x| (2..=36).contains(x)) {
        Option::Some(s) => runtime.return_1(this.to_str_radix(s).into()),
        Option::None => runtime.throw_quick(
            value_error(),
            format!(
                "int.strBase requires a radix between 2 and 36, not {}",
                value
            ),
        ),
    }
}

fn div_rem(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let other = first(args).int(runtime)?;
    if other.is_zero() {
        return runtime.throw_quick(arithmetic_error(), "Cannot divide by 0");
    }
    let (quotient, rem) = this.div_rem(&other);
    runtime.return_1(LangTuple::from_vec(vec![quotient.into(), rem.into()]).into())
}
