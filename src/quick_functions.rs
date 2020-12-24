use crate::custom_types::exceptions::{arithmetic_error, index_error};
use crate::int_var::IntVar;
use crate::operator::Operator;
use crate::rational_var::RationalVar;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FromBool, InnerVar, OptionVar, Variable};
use num::traits::Pow;
use num::{BigRational, One, ToPrimitive, Zero};

pub type QuickResult = Result<Variable, ()>;

pub fn quick_add(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            Result::Ok((IntVar::from(other) + if b { 1 } else { 0 }.into()).into())
        }
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i + IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(s)) => {
            let result = format!("{}{}", s, other.str(runtime)?);
            QuickResult::Ok(StringVar::from(result).into())
        }
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 + d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::Add, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::Add, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::Add, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_sub(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => Result::Ok(
            if b {
                IntVar::from(1) - IntVar::from(other)
            } else {
                IntVar::from(other)
            }
            .into(),
        ),
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i - IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 - d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::Subtract, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::Subtract, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::Subtract, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_u_minus(this: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => Result::Ok(IntVar::from_bool(b).into()),
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((-i).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d)) => Result::Ok((-d).into()),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::USubtract, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::USubtract, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::USubtract, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_mul(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            Result::Ok(if b { IntVar::from(other) } else { Zero::zero() }.into())
        }
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i * IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(s)) => {
            let big_var = IntVar::from(other);
            let result = match big_var.to_usize() {
                Option::Some(val) => val,
                Option::None => return mul_err(big_var, runtime),
            };
            Result::Ok(StringVar::from(s.repeat(result)).into())
        }
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 * d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::Multiply, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::Multiply, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::Multiply, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
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
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => Result::Ok(
            RationalVar::from(BigRational::new(
                if b { 1 } else { 0 }.into(),
                IntVar::from(other).into(),
            ))
            .into(),
        ),
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok(
            RationalVar::from(BigRational::new(i.into(), IntVar::from(other).into())).into(),
        ),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 / d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::Divide, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::Divide, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::Divide, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_floor_div(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => Result::Ok(
            if b {
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
            }
            .into(),
        ),
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i / IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok(IntVar::from((d1 / d2).to_integer()).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::FloorDiv, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::FloorDiv, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::FloorDiv, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_mod(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => Result::Ok(IntVar::from_bool(b).into()),
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i % IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 % d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::Modulo, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::Modulo, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::Modulo, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_subscript(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Bigint(_)) => unimplemented!(),
        Variable::Normal(InnerVar::String(val)) => {
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
        Variable::Normal(InnerVar::Decimal(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::GetAttr, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::GetAttr, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::GetAttr, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_power(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            IntVar::from(other); // Since this can be only 1 or 0, no
            Result::Ok(IntVar::from(if b { 1 } else { 0 }).into())
        }
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok(i.pow(other.into()).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::Power, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::Power, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::Power, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_left_bitshift(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            q_lshift(IntVar::from_bool(b), other.into(), runtime)
        }
        Variable::Normal(InnerVar::Bigint(i)) => q_lshift(i, other.into(), runtime),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::LeftBitshift, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::LeftBitshift, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::LeftBitshift, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

fn q_lshift(this: IntVar, other: IntVar, runtime: &mut Runtime) -> QuickResult {
    let other_usize = shift_to_usize(other, runtime)?;
    Result::Ok((this << other_usize).into())
}

pub fn quick_right_bitshift(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            q_rshift(IntVar::from_bool(b), other.into(), runtime)
        }
        Variable::Normal(InnerVar::Bigint(i)) => q_rshift(i, other.into(), runtime),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::RightBitshift, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::RightBitshift, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::RightBitshift, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

fn q_rshift(this: IntVar, other: IntVar, runtime: &mut Runtime) -> QuickResult {
    let other_usize = shift_to_usize(other, runtime)?;
    Result::Ok((this >> other_usize).into())
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
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => Result::Ok((b & other.into_bool(runtime)?).into()),
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i & IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::BitwiseAnd, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::BitwiseAnd, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::BitwiseAnd, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_bitwise_or(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => Result::Ok((b | other.into_bool(runtime)?).into()),
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i | IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::BitwiseOr, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::BitwiseOr, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::BitwiseOr, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_bitwise_xor(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => Result::Ok((b ^ other.into_bool(runtime)?).into()),
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i ^ IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::BitwiseXor, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::BitwiseXor, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::BitwiseXor, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_bitwise_not(this: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => QuickResult::Ok((!b).into()),
        Variable::Normal(InnerVar::Bigint(i)) => QuickResult::Ok((!i).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::BitwiseNot, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::BitwiseNot, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::BitwiseNot, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_equals(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::Equals, Vec::new(), runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::Equals, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::Equals, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(i, o) => {
            if let Variable::Option(i2, o2) = other {
                match (OptionVar(i, o).into(), OptionVar(i2, o2).into()) {
                    (Option::Some(this), Option::Some(other)) => quick_equals(this, other, runtime),
                    (Option::None, Option::None) => QuickResult::Ok(true.into()),
                    _ => QuickResult::Ok(false.into()),
                }
            } else {
                QuickResult::Ok(false.into())
            }
        }
        Variable::Normal(InnerVar::Tuple(t)) => {
            if let Variable::Normal(InnerVar::Tuple(t2)) = other {
                if t.len() != t2.len() {
                    return QuickResult::Ok(false.into());
                }
                for (x, y) in t.iter().zip(&t2) {
                    if !x.equals(y.clone(), runtime)? {
                        return QuickResult::Ok(false.into());
                    }
                }
                QuickResult::Ok(true.into())
            } else {
                QuickResult::Ok(false.into())
            }
        }
        _ => QuickResult::Ok((this == other).into()),
    }
}

pub fn quick_less_than(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            Result::Ok((IntVar::from(if b { 1 } else { 0 }) < IntVar::from(other)).into())
        }
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i < IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 < d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::LessThan, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::LessThan, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::LessThan, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_greater_than(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            Result::Ok((IntVar::from(if b { 1 } else { 0 }) > IntVar::from(other)).into())
        }
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i > IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 > d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::GreaterThan, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::GreaterThan, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::GreaterThan, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_less_equal(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            Result::Ok((IntVar::from(if b { 1 } else { 0 }) <= IntVar::from(other)).into())
        }
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i <= IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 <= d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::LessEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::LessEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::LessEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}

pub fn quick_greater_equal(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    match this {
        Variable::Normal(InnerVar::Null()) => unimplemented!(),
        Variable::Normal(InnerVar::Bool(b)) => {
            Result::Ok((IntVar::from(if b { 1 } else { 0 }) >= IntVar::from(other)).into())
        }
        Variable::Normal(InnerVar::Bigint(i)) => Result::Ok((i >= IntVar::from(other)).into()),
        Variable::Normal(InnerVar::String(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Decimal(d1)) => {
            if let Variable::Normal(InnerVar::Decimal(d2)) = other {
                QuickResult::Ok((d1 >= d2).into())
            } else {
                unimplemented!()
            }
        }
        Variable::Normal(InnerVar::Char(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Type(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Standard(v)) => {
            v.call_operator(Operator::GreaterEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Tuple(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Method(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Function(_)) => unimplemented!(),
        Variable::Normal(InnerVar::Custom(c)) => {
            c.into_inner()
                .call_op(Operator::GreaterEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Normal(InnerVar::Union(u)) => {
            u.call_operator(Operator::GreaterEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop_return())
        }
        Variable::Option(_, _) => unimplemented!(),
    }
}
