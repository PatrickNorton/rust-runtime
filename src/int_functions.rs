use std::boxed::Box;
use std::vec::Vec;

use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::variable::{FnResult, Variable};
use num::bigint::{BigInt, BigUint};
use num::traits::Pow;
use num::{BigRational, Signed, ToPrimitive, Zero};

pub fn get_operator(this: &BigInt, o: Operator) -> Variable {
    let func = match o {
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
        _ => unimplemented!("Operator::{:?} unimplemented", o),
    };
    Variable::Method(Box::new(StdMethod::new(
        this.clone(),
        InnerMethod::Native(func),
    )))
}

fn add(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum += BigInt::from(arg)
    }
    runtime.push(Variable::Bigint(sum));
    FnResult::Ok(())
}

fn sub(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut diff = this.clone();
    for arg in args {
        diff -= BigInt::from(arg)
    }
    runtime.push(Variable::Bigint(diff));
    FnResult::Ok(())
}

fn u_minus(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bigint(-this.clone()));
    FnResult::Ok(())
}

fn mul(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut prod = this.clone();
    for arg in args {
        prod *= BigInt::from(arg)
    }
    runtime.push(Variable::Bigint(prod));
    FnResult::Ok(())
}

fn floor_div(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = this.clone();
    for arg in args {
        ratio /= BigInt::from(arg)
    }
    runtime.push(Variable::Bigint(ratio));
    FnResult::Ok(())
}

fn div(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut ratio = BigRational::from_integer(this.clone());
    for arg in args {
        ratio /= BigRational::from_integer(BigInt::from(arg))
    }
    runtime.push(Variable::Decimal(ratio));
    FnResult::Ok(())
}

fn pow(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let arg_int = BigInt::from(args[0].clone());
    let result = this.pow(arg_int.to_biguint().unwrap_or_else(BigUint::zero));
    runtime.push(Variable::Bigint(result));
    FnResult::Ok(())
}

fn modulo(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let arg_int = BigInt::from(args[0].clone());
    runtime.push(Variable::Bigint(this.clone() % &arg_int));
    FnResult::Ok(())
}

fn eq(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if BigInt::from(arg) != *this {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn less_than(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this >= BigInt::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn greater_than(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this <= BigInt::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn less_equal(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this > BigInt::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn greater_equal(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if *this < BigInt::from(arg) {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}

fn left_bs(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let big_value = BigInt::from(args[0].clone());
    let result = match big_value.to_usize() {
        Option::None => {
            let msg = if big_value.is_negative() {
                format!("Cannot shift by {} (min shift value is 0)", big_value)
            } else {
                format!(
                    "Cannot shift by {} (max shift value is {})",
                    big_value,
                    std::usize::MAX
                )
            };
            return runtime.throw_quick(Type::String, msg.into());
        }
        Option::Some(b) => this << b,
    };
    runtime.push(Variable::Bigint(result));
    FnResult::Ok(())
}

fn right_bs(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let big_value = BigInt::from(args[0].clone());
    let result = match big_value.to_usize() {
        Option::None => {
            let msg = if big_value.is_negative() {
                format!("Cannot shift by {} (min shift value is 0)", big_value)
            } else {
                format!(
                    "Cannot shift by {} (max shift value is {})",
                    big_value,
                    std::usize::MAX
                )
            };
            return runtime.throw_quick(Type::String, msg.into());
        }
        Option::Some(b) => this >> b,
    };
    runtime.push(Variable::Bigint(result));
    FnResult::Ok(())
}

fn bitwise_and(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum &= &BigInt::from(arg)
    }
    runtime.push(Variable::Bigint(sum));
    FnResult::Ok(())
}

fn bitwise_or(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum |= &BigInt::from(arg)
    }
    runtime.push(Variable::Bigint(sum));
    FnResult::Ok(())
}

fn bitwise_not(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bigint(!this.clone()));
    FnResult::Ok(())
}

fn bitwise_xor(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut sum = this.clone();
    for arg in args {
        sum ^= &BigInt::from(arg)
    }
    runtime.push(Variable::Bigint(sum));
    FnResult::Ok(())
}

fn to_str(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::String(this.to_str_radix(10).into()));
    FnResult::Ok(())
}

fn to_int(this: &BigInt, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bigint(this.clone()));
    FnResult::Ok(())
}
