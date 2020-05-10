use num::bigint::{ToBigInt, ToBigUint};
use num::traits::{abs, FromPrimitive, Num, One, Signed};
use num::{BigInt, BigUint, ToPrimitive, Zero};
use num_traits::Pow;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, Shr, Sub, SubAssign,
};
use std::rc::Rc;
use std::str::FromStr;

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum IntVar {
    Small(isize),
    Big(Rc<BigInt>),
}

impl IntVar {
    pub fn to_str_radix(&self, radix: u32) -> String {
        match self {
            IntVar::Small(_) => unimplemented!(),
            IntVar::Big(b) => b.to_str_radix(radix),
        }
    }
}

impl From<BigInt> for IntVar {
    fn from(x: BigInt) -> Self {
        match x.to_isize() {
            Option::Some(s) => IntVar::Small(s),
            Option::None => IntVar::Big(Rc::new(x)),
        }
    }
}

impl From<Rc<BigInt>> for IntVar {
    fn from(x: Rc<BigInt>) -> Self {
        match x.to_isize() {
            Option::Some(s) => IntVar::Small(s),
            Option::None => IntVar::Big(x),
        }
    }
}

macro_rules! from_prim_impl {
    ($fn_name:ident, $typ:ty) => {
        fn $fn_name(n: $typ) -> Option<Self> {
            match isize::try_from(n) {
                Result::Ok(v) => Option::Some(v.into()),
                Result::Err(_) => Option::Some(BigInt::from(n).into()),
            }
        }
    };
}

impl FromPrimitive for IntVar {
    from_prim_impl!(from_i64, i64);
    from_prim_impl!(from_u64, u64);
    from_prim_impl!(from_u128, u128);
    from_prim_impl!(from_i128, i128);
    from_prim_impl!(from_usize, usize);
    from_prim_impl!(from_isize, isize);
}

macro_rules! to_prim_impl {
    ($fn_name:ident, $typ:ty) => {
        fn $fn_name(&self) -> Option<$typ> {
            match self {
                IntVar::Small(i) => (*i).try_into().ok(),
                IntVar::Big(b) => b.$fn_name(),
            }
        }
    };
}

impl ToPrimitive for IntVar {
    to_prim_impl!(to_i64, i64);
    to_prim_impl!(to_u64, u64);
    to_prim_impl!(to_u128, u128);
    to_prim_impl!(to_i128, i128);
    to_prim_impl!(to_usize, usize);
    to_prim_impl!(to_isize, isize);
}

macro_rules! impl_from {
    ($typ:ty) => {
        impl From<$typ> for IntVar {
            fn from(x: $typ) -> Self {
                match isize::try_from(x) {
                    Result::Ok(v) => v.into(),
                    Result::Err(_) => IntVar::Big(Rc::new(BigInt::from(x))),
                }
            }
        }
    };
}

impl_from!(u8);
impl_from!(i8);
impl_from!(u16);
impl_from!(i16);
impl_from!(u32);
impl_from!(i32);
impl_from!(u64);
impl_from!(i64);
impl_from!(u128);
impl_from!(i128);
impl_from!(usize);

impl From<isize> for IntVar {
    fn from(x: isize) -> Self {
        IntVar::Small(x)
    }
}

impl From<IntVar> for BigInt {
    fn from(x: IntVar) -> Self {
        match x {
            IntVar::Small(i) => i.into(),
            IntVar::Big(b) => (*b).clone(),
        }
    }
}

impl FromStr for IntVar {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match isize::from_str(s) {
            Result::Ok(i) => Result::Ok(IntVar::Small(i)),
            Result::Err(_) => Result::Ok(IntVar::Big(Rc::new(
                BigInt::from_str(s).or(Result::Err(()))?,
            ))),
        }
    }
}

impl ToBigInt for IntVar {
    fn to_bigint(&self) -> Option<BigInt> {
        match self {
            IntVar::Small(i) => Option::Some(BigInt::from(*i)),
            IntVar::Big(b) => Option::Some((**b).clone()),
        }
    }
}

impl ToBigUint for IntVar {
    fn to_biguint(&self) -> Option<BigUint> {
        self.to_bigint()?.to_biguint()
    }
}

impl Num for IntVar {
    type FromStrRadixErr = ();

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        match isize::from_str_radix(str, radix) {
            Result::Ok(val) => Result::Ok(val.into()),
            Result::Err(_) => Result::Ok(IntVar::Big(Rc::new(
                BigInt::from_str_radix(str, radix).or(Result::Err(()))?,
            ))),
        }
    }
}

impl Default for IntVar {
    fn default() -> Self {
        Zero::zero()
    }
}

impl Zero for IntVar {
    fn zero() -> Self {
        IntVar::Small(0)
    }

    fn is_zero(&self) -> bool {
        match self {
            IntVar::Small(s) => *s == 0,
            IntVar::Big(b) => b.is_zero(),
        }
    }
}

impl One for IntVar {
    fn one() -> Self {
        IntVar::Small(1)
    }
}

impl Signed for IntVar {
    fn abs(&self) -> Self {
        match self {
            IntVar::Small(i) => abs(*i).into(),
            IntVar::Big(b) => IntVar::Big(Rc::new(b.abs())),
        }
    }

    fn abs_sub(&self, _other: &Self) -> Self {
        unimplemented!()
    }

    fn signum(&self) -> Self {
        match self {
            IntVar::Small(i) => Signed::signum(i).into(),
            IntVar::Big(b) => b.signum().into(),
        }
    }

    fn is_positive(&self) -> bool {
        match self {
            IntVar::Small(i) => *i > 0,
            IntVar::Big(b) => b.is_positive(),
        }
    }

    fn is_negative(&self) -> bool {
        match self {
            IntVar::Small(i) => *i < 0,
            IntVar::Big(b) => b.is_negative(),
        }
    }
}

impl Add for IntVar {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            IntVar::Small(s1) => match rhs {
                IntVar::Small(s2) => match s1.checked_add(s2) {
                    Option::Some(val) => IntVar::Small(val),
                    Option::None => (BigInt::from(s1) + s2).into(),
                },
                IntVar::Big(b2) => (&*b2 + s1).into(),
            },
            IntVar::Big(b1) => match rhs {
                IntVar::Small(s2) => (&*b1 + s2).into(),
                IntVar::Big(b2) => (&*b1 + &*b2).into(),
            },
        }
    }
}

impl Add for &IntVar {
    type Output = IntVar;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            IntVar::Small(s1) => match rhs {
                IntVar::Small(s2) => match s1.checked_add(*s2) {
                    Option::Some(val) => IntVar::Small(val),
                    Option::None => (BigInt::from(*s1) + *s2).into(),
                },
                IntVar::Big(b2) => (&**b2 + *s1).into(),
            },
            IntVar::Big(b1) => match rhs {
                IntVar::Small(s2) => (&**b1 + *s2).into(),
                IntVar::Big(b2) => (&**b1 + &**b2).into(),
            },
        }
    }
}

impl AddAssign for IntVar {
    fn add_assign(&mut self, rhs: Self) {
        *self = (self as &Self) + &rhs;
    }
}

impl Sub for IntVar {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            IntVar::Small(s1) => match rhs {
                IntVar::Small(s2) => match s1.checked_sub(s2) {
                    Option::Some(val) => IntVar::Small(val),
                    Option::None => (BigInt::from(s1) - s2).into(),
                },
                IntVar::Big(b2) => (&*b2 - s1).into(),
            },
            IntVar::Big(b1) => match rhs {
                IntVar::Small(s2) => (&*b1 - s2).into(),
                IntVar::Big(b2) => (&*b1 - &*b2).into(),
            },
        }
    }
}

impl Sub for &IntVar {
    type Output = IntVar;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            IntVar::Small(s1) => match rhs {
                IntVar::Small(s2) => match s1.checked_sub(*s2) {
                    Option::Some(val) => IntVar::Small(val),
                    Option::None => (BigInt::from(*s1) - s2).into(),
                },
                IntVar::Big(b2) => (&**b2 - *s1).into(),
            },
            IntVar::Big(b1) => match rhs {
                IntVar::Small(s2) => (&**b1 - *s2).into(),
                IntVar::Big(b2) => (&**b1 - &**b2).into(),
            },
        }
    }
}

impl SubAssign for IntVar {
    fn sub_assign(&mut self, rhs: Self) {
        *self = (self as &Self) - &rhs;
    }
}

impl Neg for IntVar {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            IntVar::Small(i) => (-i).into(),
            IntVar::Big(b) => (-(*b).clone()).into(),
        }
    }
}

impl Mul for IntVar {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            IntVar::Small(s1) => match rhs {
                IntVar::Small(s2) => match s1.checked_mul(s2) {
                    Option::Some(val) => IntVar::Small(val),
                    Option::None => (BigInt::from(s1) * s2).into(),
                },
                IntVar::Big(b2) => {
                    let mut result = (*b2).clone();
                    result *= s1;
                    result.into()
                }
            },
            IntVar::Big(b1) => match rhs {
                IntVar::Small(s2) => {
                    let mut result = (*b1).clone();
                    result *= s2;
                    result.into()
                }
                IntVar::Big(b2) => (&*b1 * &*b2).into(),
            },
        }
    }
}

impl MulAssign for IntVar {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}

impl Div for IntVar {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            IntVar::Small(s1) => match rhs {
                // No need for check here, div can't overflow
                IntVar::Small(s2) => IntVar::Small(s1 / s2),
                IntVar::Big(b2) => {
                    let mut result = (*b2).clone();
                    result /= s1;
                    result.into()
                }
            },
            IntVar::Big(b1) => match rhs {
                IntVar::Small(s2) => {
                    let mut result = (*b1).clone();
                    result /= s2;
                    result.into()
                }
                IntVar::Big(b2) => (&*b1 / &*b2).into(),
            },
        }
    }
}

impl DivAssign for IntVar {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.clone() / rhs;
    }
}

impl Rem for IntVar {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            IntVar::Small(s1) => match rhs {
                // No need for check here, div can't overflow
                IntVar::Small(s2) => IntVar::Small(s1 % s2),
                IntVar::Big(b2) => {
                    let mut result = (*b2).clone();
                    result %= s1;
                    result.into()
                }
            },
            IntVar::Big(b1) => match rhs {
                IntVar::Small(s2) => {
                    let mut result = (*b1).clone();
                    result %= s2;
                    result.into()
                }
                IntVar::Big(b2) => (&*b1 % &*b2).into(),
            },
        }
    }
}

impl RemAssign for IntVar {
    fn rem_assign(&mut self, rhs: Self) {
        *self = self.clone() % rhs;
    }
}

impl Pow<Self> for IntVar {
    type Output = Self;

    fn pow(self, rhs: Self) -> Self::Output {
        if self.is_negative() {
            panic!("Cannot 'pow' with negative number");
        }
        self.to_bigint()
            .unwrap()
            .pow(rhs.to_biguint().unwrap())
            .into()
    }
}

impl Shl<usize> for IntVar {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        (self.to_bigint().unwrap() << rhs).into()
    }
}

impl Shr<usize> for IntVar {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        (self.to_bigint().unwrap() >> rhs).into()
    }
}

macro_rules! impl_bit {
    ($trait_name:ty, $fn_name:ident) => {
        impl $trait_name for IntVar {
            type Output = Self;

            fn $fn_name(self, rhs: Self) -> Self::Output {
                match self {
                    IntVar::Small(s1) => match rhs {
                        IntVar::Small(s2) => s1.$fn_name(s2).into(),
                        IntVar::Big(b2) => (*b2).clone().$fn_name(&s1.into()).into(),
                    },
                    IntVar::Big(b1) => match rhs {
                        IntVar::Small(s2) => (*b1).clone().$fn_name(&s2.into()).into(),
                        IntVar::Big(b2) => (*b1).clone().$fn_name(&*b2).into(),
                    },
                }
            }
        }
    };
}

impl_bit!(BitAnd, bitand);
impl_bit!(BitOr, bitor);
impl_bit!(BitXor, bitxor);

impl BitAndAssign for IntVar {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.clone() & rhs;
    }
}

impl BitOrAssign for IntVar {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.clone() | rhs;
    }
}

impl BitXorAssign for IntVar {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.clone() ^ rhs;
    }
}

impl Not for IntVar {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            IntVar::Small(i) => (!i).into(),
            IntVar::Big(b) => (!&*b).into(),
        }
    }
}

impl Display for IntVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IntVar::Small(i) => std::fmt::Display::fmt(i, f),
            IntVar::Big(b) => std::fmt::Display::fmt(b, f),
        }
    }
}
