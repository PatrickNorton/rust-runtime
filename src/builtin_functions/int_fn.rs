use crate::custom_types::exceptions::{arithmetic_error, value_error};
use crate::int_var::IntVar;
use crate::method::{InnerMethod, NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::rational_var::RationalVar;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};
use num::traits::Pow;
use num::{Signed, ToPrimitive, Zero};
use std::boxed::Box;
use std::mem::take;
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
        _ => unimplemented!("int.{} unimplemented", o.name()),
    }
}

pub fn get_operator(this: IntVar, o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

fn add(this: &IntVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        return runtime.return_1((this + &take(&mut args[0]).into()).into());
    }
    let mut sum = this.clone();
    for arg in args {
        sum += IntVar::from(arg)
    }
    runtime.return_1(Variable::Bigint(sum))
}

fn sub(this: &IntVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        return runtime.return_1((this - &take(&mut args[0]).into()).into());
    }
    let mut diff = this.clone();
    for arg in args {
        diff -= IntVar::from(arg)
    }
    runtime.return_1(Variable::Bigint(diff))
}

fn u_minus(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Bigint(this.neg()))
}

fn mul(this: &IntVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        return runtime.return_1((this * &take(&mut args[0]).into()).into());
    }
    let mut prod = this.clone();
    for arg in args {
        prod *= IntVar::from(arg)
    }
    runtime.return_1(Variable::Bigint(prod))
}

fn floor_div(this: &IntVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        let var: IntVar = take(&mut args[0]).into();
        if var.is_zero() {
            return runtime.throw_quick(value_error(), "Cannot divide by 0".into());
        }
        return runtime.return_1((this / &var).into());
    }
    let mut ratio = this.clone();
    for arg in args {
        let var = IntVar::from(arg);
        if var.is_zero() {
            return runtime.throw_quick(value_error(), "Cannot divide by 0".into());
        }
        ratio /= var;
    }
    runtime.return_1(Variable::Bigint(ratio))
}

fn div(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = RationalVar::from_integer(this.clone().into());
    for arg in args {
        let var = IntVar::from(arg);
        if var.is_zero() {
            return runtime.throw_quick(value_error(), "Cannot divide by 0".into());
        }
        ratio /= RationalVar::from_integer(var.into())
    }
    runtime.return_1(Variable::Decimal(ratio))
}

fn pow(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let arg_int = IntVar::from(args[0].clone());
    let result = this.clone().pow(arg_int);
    runtime.return_1(Variable::Bigint(result))
}

fn modulo(this: &IntVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let arg_int = IntVar::from(take(&mut args[0]));
    if arg_int.is_zero() {
        runtime.throw_quick(value_error(), "Cannot modulo by 0".into())
    } else {
        runtime.return_1(Variable::Bigint(this % &arg_int))
    }
}

fn eq(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if IntVar::from(arg) != *this {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn less_than(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this >= IntVar::from(arg) {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn greater_than(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this <= IntVar::from(arg) {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn less_equal(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this > IntVar::from(arg) {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn greater_equal(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this < IntVar::from(arg) {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn left_bs(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let big_value = IntVar::from(args[0].clone());
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
            return runtime.throw_quick(arithmetic_error(), msg.into());
        }
        Option::Some(b) => this << b,
    };
    runtime.return_1(Variable::Bigint(result))
}

fn right_bs(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let big_value = IntVar::from(args[0].clone());
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
            return runtime.throw_quick(arithmetic_error(), msg.into());
        }
        Option::Some(b) => this >> b,
    };
    runtime.return_1(Variable::Bigint(result))
}

fn bitwise_and(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum &= IntVar::from(arg)
    }
    runtime.return_1(Variable::Bigint(sum))
}

fn bitwise_or(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum |= IntVar::from(arg)
    }
    runtime.return_1(Variable::Bigint(sum))
}

fn bitwise_not(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Bigint(!this.clone()))
}

fn bitwise_xor(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum ^= IntVar::from(arg)
    }
    runtime.return_1(Variable::Bigint(sum))
}

fn to_str(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String(this.to_string().into()))
}

fn to_int(this: &IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Bigint(this.clone()))
}
