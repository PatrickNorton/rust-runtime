use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::Variable;
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
            v.call_operator(Operator::Add, vec![other], runtime)?;
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
            v.call_operator(Operator::Subtract, vec![other], runtime)?;
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
            v.call_operator(Operator::USubtract, Vec::new(), runtime)?;
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
            v.call_operator(Operator::Multiply, vec![other], runtime)?;
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
            v.call_operator(Operator::Divide, vec![other], runtime)?;
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
            v.call_operator(Operator::FloorDiv, vec![other], runtime)?;
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
            v.call_operator(Operator::Modulo, vec![other], runtime)?;
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
            v.call_operator(Operator::GetAttr, vec![other], runtime)?;
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
            v.call_operator(Operator::Power, vec![other], runtime)?;
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
            v.call_operator(Operator::LeftBitshift, vec![other], runtime)?;
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
            v.call_operator(Operator::RightBitshift, vec![other], runtime)?;
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
            v.call_operator(Operator::BitwiseAnd, vec![other], runtime)?;
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
            v.call_operator(Operator::BitwiseOr, vec![other], runtime)?;
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
            v.call_operator(Operator::BitwiseXor, vec![other], runtime)?;
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
            v.call_operator(Operator::BitwiseNot, Vec::new(), runtime)?;
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
            v.call_operator(Operator::Equals, Vec::new(), runtime)?;
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
            v.call_operator(Operator::LessThan, vec![other], runtime)?;
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
            v.call_operator(Operator::GreaterThan, vec![other], runtime)?;
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
            v.call_operator(Operator::LessEqual, vec![other], runtime)?;
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
            v.call_operator(Operator::GreaterEqual, vec![other], runtime)?;
            QuickResult::Ok(runtime.pop())
        }
        Variable::Method(_) => unimplemented!(),
        Variable::Function(_) => unimplemented!(),
        Variable::Custom(_) => unimplemented!(),
    };
}