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
        Operator::Hash => hash,
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
        return runtime.return_1((this + IntVar::from(first(args))).into());
    }
    let mut sum = this;
    for arg in args {
        sum += IntVar::from(arg)
    }
    runtime.return_1(sum.into())
}

fn sub(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    if args.len() == 1 {
        return runtime.return_1((this - IntVar::from(first(args))).into());
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
        return runtime.return_1((this * IntVar::from(first(args))).into());
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
            return runtime.throw_quick(arithmetic_error(), "Cannot divide by zero");
        }
        return runtime.return_1((this / var).into());
    }
    let mut ratio = this;
    for arg in args {
        let var = IntVar::from(arg);
        if var.is_zero() {
            return runtime.throw_quick(arithmetic_error(), "Cannot divide by zero");
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
            return runtime.throw_quick(arithmetic_error(), "Cannot divide by zero");
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
        runtime.throw_quick(arithmetic_error(), "Cannot modulo by zero")
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

fn hash(this: IntVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.into())
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

#[cfg(test)]
mod test {
    use crate::builtin_functions::int_fn::{
        add, div, eq, floor_div, greater_than, left_bs, less_than, mul, right_bs, sub, u_minus,
    };
    use crate::int_var::IntVar;
    use crate::rational_var::RationalVar;
    use crate::runtime::Runtime;
    use num::{BigInt, BigRational, One, Zero};

    #[test]
    fn sum() {
        let a = IntVar::one();
        let b = IntVar::from(2);
        let result = Runtime::test(|runtime| add(a, vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(IntVar::from(3).into()))
    }

    #[test]
    fn diff() {
        let a = IntVar::one();
        let b = IntVar::from(2);
        let result = Runtime::test(|runtime| sub(a, vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(IntVar::from(-1).into()))
    }

    #[test]
    fn inverse() {
        let a = IntVar::one();
        let result = Runtime::test(|runtime| u_minus(a, vec![], runtime));
        assert_eq!(result, Result::Ok(IntVar::from(-1).into()))
    }

    #[test]
    fn prod() {
        let a = IntVar::one();
        let b = IntVar::from(2);
        let result = Runtime::test(|runtime| mul(a, vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(IntVar::from(2).into()))
    }

    #[test]
    fn floor_quot() {
        let a = IntVar::one();
        let b = IntVar::from(2);
        let result = Runtime::test(|runtime| floor_div(a, vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(IntVar::zero().into()))
    }

    #[test]
    fn quot() {
        let a = IntVar::one();
        let b = IntVar::from(2);
        let result = Runtime::test(|runtime| div(a, vec![b.into()], runtime));
        assert_eq!(
            result,
            Result::Ok(RationalVar::from(BigRational::new(BigInt::one(), BigInt::from(2))).into())
        )
    }

    #[test]
    fn equal() {
        let a = IntVar::one();
        let b = a.clone();
        let c = IntVar::zero();
        let d = IntVar::from(2);
        let result = Runtime::test(|runtime| eq(a.clone(), vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(true.into()));
        let result = Runtime::test(|runtime| eq(a.clone(), vec![c.into()], runtime));
        assert_eq!(result, Result::Ok(false.into()));
        let result = Runtime::test(|runtime| eq(a, vec![d.into()], runtime));
        assert_eq!(result, Result::Ok(false.into()));
    }

    #[test]
    fn lt() {
        let a = IntVar::one();
        let b = a.clone();
        let c = IntVar::zero();
        let d = IntVar::from(2);
        let result = Runtime::test(|runtime| less_than(a.clone(), vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(false.into()));
        let result = Runtime::test(|runtime| less_than(a.clone(), vec![c.into()], runtime));
        assert_eq!(result, Result::Ok(false.into()));
        let result = Runtime::test(|runtime| less_than(a, vec![d.into()], runtime));
        assert_eq!(result, Result::Ok(true.into()));
    }

    #[test]
    fn gt() {
        let a = IntVar::one();
        let b = a.clone();
        let c = IntVar::zero();
        let d = IntVar::from(2);
        let result = Runtime::test(|runtime| greater_than(a.clone(), vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(false.into()));
        let result = Runtime::test(|runtime| greater_than(a.clone(), vec![c.into()], runtime));
        assert_eq!(result, Result::Ok(true.into()));
        let result = Runtime::test(|runtime| greater_than(a, vec![d.into()], runtime));
        assert_eq!(result, Result::Ok(false.into()));
    }

    #[test]
    fn right_shift() {
        let a = IntVar::from(8);
        let b = IntVar::from(2);
        let result = Runtime::test(|runtime| right_bs(a, vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(IntVar::from(2).into()))
    }

    #[test]
    fn left_shift() {
        let a = IntVar::from(8);
        let b = IntVar::from(2);
        let result = Runtime::test(|runtime| left_bs(a, vec![b.into()], runtime));
        assert_eq!(result, Result::Ok(IntVar::from(32).into()))
    }
}
