use crate::custom_types::exceptions::{arithmetic_error, index_error};
use crate::int_var::IntVar;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FromBool, Variable};
use num::traits::Pow;
use num::{BigRational, One, ToPrimitive, Zero};

pub type QuickResult = Result<Variable, ()>;

pub fn quick_add(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            IntVar::from(other) + if b { 1 } else { 0 }.into(),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i + IntVar::from(other))),
        Variable::String(s) => {
            let result = format!("{}{}", s, other.str(runtime)?);
            QuickResult::Ok(Variable::String(result.into()))
        }
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 + d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::Add, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_sub(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(if b {
            IntVar::from(1) - IntVar::from(other)
        } else {
            IntVar::from(other)
        })),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i - IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 - d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::Subtract, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_u_minus(this: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(IntVar::from_bool(b))),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(-i)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d) => Result::Ok(Variable::Decimal(-d)),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::USubtract, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_mul(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(if b {
            other.into()
        } else {
            Zero::zero()
        })),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i * IntVar::from(other))),
        Variable::String(s) => {
            let big_var = IntVar::from(other);
            let result = match big_var.to_usize() {
                Option::Some(val) => val,
                Option::None => return mul_err(big_var, runtime),
            };
            Result::Ok(Variable::String(s.repeat(result).into()))
        }
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 * d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::Multiply, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

fn mul_err(big_var: IntVar, runtime: &mut Runtime) -> QuickResult {
    runtime.throw_quick(
        arithmetic_error(),
        format!(
            "Too many string repetitions: max number of shifts \
                for a non-empty string is {}, attempted to shift by {}",
            usize::MAX,
            big_var,
        )
        .into(),
    )?;
    unreachable!()
}

pub fn quick_div(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Decimal(
            BigRational::new(if b { 1 } else { 0 }.into(), IntVar::from(other).into()).into(),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Decimal(
            BigRational::new(i.into(), IntVar::from(other).into()).into(),
        )),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 / d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::Divide, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_floor_div(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(if b {
            let var = IntVar::from(other);
            if var.is_one() {
                One::one()
            } else if (-var).is_one() {
                -IntVar::one()
            } else {
                Zero::zero()
            }
        } else {
            Zero::zero()
        })),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i / IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bigint((d1 / d2).to_integer().into()))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::FloorDiv, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_mod(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(IntVar::from_bool(b))),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i % IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 % d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::Modulo, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_subscript(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(_) => unimplemented!(),
        Variable::Bigint(_) => unimplemented!(),
        Variable::String(val) => {
            let index = IntVar::from(other).to_usize().unwrap();
            match val.chars().nth(index) {
                Option::None => {
                    runtime.push_native();
                    runtime.throw_quick(index_error(), "Index out of bounds".into())?;
                    unreachable!() // Native frame will always return FnResult::Err
                }
                Option::Some(value) => Result::Ok(value.into()),
            }
        }
        Variable::Decimal(_) => unimplemented!(),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::GetAttr, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_power(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => {
            IntVar::from(other); // Since this can be only 1 or 0, no
            Result::Ok(Variable::Bigint(if b { 1 } else { 0 }.into()))
        }
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i.pow(IntVar::from(other)))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::Power, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_left_bitshift(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => q_lshift(IntVar::from_bool(b), other.into(), runtime),
        Variable::Bigint(i) => q_lshift(i, other.into(), runtime),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::LeftBitshift, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

fn q_lshift(this: IntVar, other: IntVar, runtime: &mut Runtime) -> QuickResult {
    let other_usize = shift_to_usize(other, runtime)?;
    Result::Ok(Variable::Bigint(this << other_usize))
}

pub fn quick_right_bitshift(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => q_rshift(IntVar::from_bool(b), other.into(), runtime),
        Variable::Bigint(i) => q_rshift(i, other.into(), runtime),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::RightBitshift, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

fn q_rshift(this: IntVar, other: IntVar, runtime: &mut Runtime) -> QuickResult {
    let other_usize = shift_to_usize(other, runtime)?;
    Result::Ok(Variable::Bigint(this >> other_usize))
}

fn shift_to_usize(big_var: IntVar, runtime: &mut Runtime) -> Result<usize, ()> {
    Result::Ok(match big_var.to_usize() {
        Option::Some(val) => val,
        Option::None => {
            runtime.throw_quick(arithmetic_error(), shift_err(big_var))?;
            unreachable!()
        }
    })
}

fn shift_err(big_val: IntVar) -> StringVar {
    format!(
        "Attempted bitshift of {}, which is more than the max allowed shift {}",
        big_val,
        usize::MAX
    )
    .into()
}

pub fn quick_bitwise_and(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(b & other.to_bool(runtime)?)),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i & IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::BitwiseAnd, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_bitwise_or(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(b | other.to_bool(runtime)?)),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i | IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::BitwiseOr, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_bitwise_xor(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(b ^ other.to_bool(runtime)?)),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i ^ IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::BitwiseXor, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_bitwise_not(this: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => QuickResult::Ok(Variable::Bool(!b)),
        Variable::Bigint(i) => QuickResult::Ok(Variable::Bigint(!i)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::BitwiseNot, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_equals(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Standard(v) => {
            v.call_operator(Operator::Equals, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Custom(c) => {
            (*c).clone()
                .call_op(Operator::Equals, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        _ => QuickResult::Ok(Variable::Bool(this == other)),
    }
}

pub fn quick_less_than(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(
            IntVar::from(if b { 1 } else { 0 }) < IntVar::from(other),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bool(i < IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bool(d1 < d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::LessThan, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_greater_than(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(
            IntVar::from(if b { 1 } else { 0 }) > IntVar::from(other),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bool(i > IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bool(d1 > d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::GreaterThan, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_less_equal(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(
            IntVar::from(if b { 1 } else { 0 }) <= IntVar::from(other),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bool(i <= IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bool(d1 <= d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::LessEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}

pub fn quick_greater_equal(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(
            IntVar::from(if b { 1 } else { 0 }) >= IntVar::from(other),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bool(i >= IntVar::from(other))),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bool(d1 >= d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Char(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.call_operator(Operator::GreaterEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    }
}
