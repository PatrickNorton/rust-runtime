use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{Name, Variable};
use num::traits::Pow;
use num::{BigInt, BigRational, BigUint, FromPrimitive, ToPrimitive, Zero};
use std::rc::Rc;

pub type QuickResult = Result<Variable, ()>;

pub fn quick_add(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            other.int(runtime)? + if b { 1 } else { 0 },
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i + other.int(runtime)?)),
        Variable::String(s) => {
            let result = format!("{}{}", s, other.str(runtime)?);
            QuickResult::Ok(Variable::String(StringVar::Other(Rc::new(
                result.into_boxed_str(),
            ))))
        }
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 + d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::Add))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_sub(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            if b { 1 } else { 0 } - other.int(runtime)?,
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i - other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 - d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::Subtract))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_u_minus(this: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            BigInt::from_i8(if b { -1 } else { 0 }).unwrap(),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(-i)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d) => Result::Ok(Variable::Decimal(-d)),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::USubtract))
                .call((vec![], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_mul(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            other.int(runtime)? * if b { 1 } else { 0 },
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i * other.int(runtime)?)),
        Variable::String(s) => {
            let result = other
                .int(runtime)?
                .to_usize()
                .expect("Too many repetitions");
            Result::Ok(Variable::String(s.repeat(result).into()))
        }
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 * d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::Multiply))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_div(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Decimal(BigRational::new(
            if b { 1 } else { 0 }.into(),
            other.int(runtime)?,
        ))),
        Variable::Bigint(i) => {
            Result::Ok(Variable::Decimal(BigRational::new(i, other.int(runtime)?)))
        }
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 / d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::Divide))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_floor_div(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            if b { 1 } else { 0 } / other.int(runtime)?,
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i / other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bigint((d1 / d2).to_integer()))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::FloorDiv))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_mod(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            if b { 1 } else { 0 } % other.int(runtime)?,
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i % other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Decimal(d1 % d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::Modulo))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_subscript(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(_) => unimplemented!(),
        Variable::Bigint(_) => unimplemented!(),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::GetAttr))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_power(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => {
            other.int(runtime)?; // Since this can be only 1 or 0, no
            Result::Ok(Variable::Bigint(if b { 1 } else { 0 }.into()))
        }
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(
            i.pow(
                other
                    .int(runtime)?
                    .to_biguint()
                    .unwrap_or_else(BigUint::zero),
            ),
        )),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::Power))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_left_bitshift(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            (if b { 1 } else { 0 }
                << other
                    .int(runtime)?
                    .to_usize()
                    .expect("Value too big to shift"))
            .into(),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(
            i << other
                .int(runtime)?
                .to_usize()
                .expect("Value too big to shift"),
        )),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::LeftBitshift))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_right_bitshift(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bigint(
            (if b { 1 } else { 0 }
                >> other
                    .int(runtime)?
                    .to_usize()
                    .expect("Value too big to shift"))
            .into(),
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(
            i >> other
                .int(runtime)?
                .to_usize()
                .expect("Value too big to shift"),
        )),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::RightBitshift))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_bitwise_and(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(b & other.to_bool(runtime)?)),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i & &other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::BitwiseAnd))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_bitwise_or(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(b | other.to_bool(runtime)?)),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i | &other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::BitwiseOr))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_bitwise_xor(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(b ^ other.to_bool(runtime)?)),
        Variable::Bigint(i) => Result::Ok(Variable::Bigint(i ^ &other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::BitwiseXor))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_bitwise_not(this: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => QuickResult::Ok(Variable::Bool(!b)),
        Variable::Bigint(i) => QuickResult::Ok(Variable::Bigint(!i)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(_) => unimplemented!(),
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::BitwiseNot))
                .call((vec![], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_equals(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::Equals))
                .call((vec![], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        _ => QuickResult::Ok(Variable::Bool(this == other)),
    };
}

pub fn quick_less_than(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(
            BigInt::from(if b { 1 } else { 0 }) < other.int(runtime)?,
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bool(i < other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bool(d1 < d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::LessThan))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_greater_than(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(
            BigInt::from(if b { 1 } else { 0 }) > other.int(runtime)?,
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bool(i > other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bool(d1 > d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::GreaterThan))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_less_equal(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(
            BigInt::from(if b { 1 } else { 0 }) <= other.int(runtime)?,
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bool(i <= other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bool(d1 <= d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::LessEqual))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}

pub fn quick_greater_equal(this: Variable, other: Variable, runtime: &mut Runtime) -> QuickResult {
    return match this {
        Variable::Null() => unimplemented!(),
        Variable::Bool(b) => Result::Ok(Variable::Bool(
            BigInt::from(if b { 1 } else { 0 }) >= other.int(runtime)?,
        )),
        Variable::Bigint(i) => Result::Ok(Variable::Bool(i >= other.int(runtime)?)),
        Variable::String(_) => unimplemented!(),
        Variable::Decimal(d1) => {
            if let Variable::Decimal(d2) = other {
                QuickResult::Ok(Variable::Bool(d1 >= d2))
            } else {
                unimplemented!()
            }
        }
        Variable::Type(_) => unimplemented!(),
        Variable::Standard(v) => {
            v.index(Name::Operator(Operator::GreaterEqual))
                .call((vec![other], runtime))?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}
