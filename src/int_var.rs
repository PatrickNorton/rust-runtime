use num::bigint::{Sign, ToBigInt, ToBigUint, TryFromBigIntError};
use num::traits::{abs, FromPrimitive, Num, One, Pow, Signed};
use num::{BigInt, BigUint, Integer, ToPrimitive, Zero};
use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Binary, Display, Formatter, LowerHex, Octal, UpperHex};
use std::hash::{Hash, Hasher};
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, Shr, Sub, SubAssign,
};
use std::rc::Rc;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum IntVar {
    Small(isize),
    Big(Rc<BigInt>),
}

impl IntVar {
    pub fn to_str_radix(&self, radix: u32) -> String {
        match self {
            IntVar::Small(s) => match radix {
                10 => format!("{}", s),
                16 => format!("{:x}", s),
                8 => format!("{:o}", s),
                2 => format!("{:b}", s),
                x => BigInt::from(x).to_str_radix(radix),
            },
            IntVar::Big(b) => b.to_str_radix(radix),
        }
    }
}

pub fn normalize(len: usize, signed_index: IntVar) -> Result<usize, IntVar> {
    let index = if signed_index.is_negative() {
        signed_index + len
    } else {
        signed_index
    };
    usize::try_from(index).and_then(|a| {
        if a < len {
            Result::Ok(a)
        } else {
            Result::Err(a.into())
        }
    })
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

macro_rules! impl_try {
    ($typ:ty) => {
        impl TryFrom<IntVar> for $typ {
            type Error = IntVar;

            fn try_from(x: IntVar) -> Result<Self, Self::Error> {
                match x {
                    IntVar::Small(i) => i.try_into().map_err(|_| IntVar::Small(i)),
                    IntVar::Big(b) => b.as_ref().try_into().map_err(|_| IntVar::Big(b)),
                }
            }
        }
    };
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

impl_try!(u8);
impl_try!(i8);
impl_try!(u16);
impl_try!(i16);
impl_try!(u32);
impl_try!(i32);
impl_try!(u64);
impl_try!(i64);
impl_try!(u128);
impl_try!(i128);
impl_try!(usize);
impl_try!(isize);

impl PartialEq for IntVar {
    fn eq(&self, other: &Self) -> bool {
        match self {
            IntVar::Small(a) => match other {
                IntVar::Small(b) => *a == *b,
                IntVar::Big(b) => b.to_isize().map(|x| &x == a).unwrap_or(false),
            },
            IntVar::Big(a) => match other {
                IntVar::Small(b) => a.to_isize().map(|x| &x == b).unwrap_or(false),
                IntVar::Big(b) => *a.as_ref() == *b.as_ref(),
            },
        }
    }
}

impl Eq for IntVar {}

impl Hash for IntVar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            IntVar::Small(i) => BigInt::from(*i).hash(state),
            IntVar::Big(b) => b.as_ref().hash(state),
        }
    }
}

impl PartialOrd for IntVar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IntVar {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            IntVar::Small(a) => match other {
                IntVar::Small(b) => a.cmp(b),
                IntVar::Big(b) => match b.to_isize() {
                    Some(i) => a.cmp(&i),
                    None => match b.sign() {
                        Sign::Minus => Ordering::Greater,
                        Sign::NoSign => {
                            unreachable!("A BigInt with a sign of 0 should be convertible to isize")
                        }
                        Sign::Plus => Ordering::Less,
                    },
                },
            },
            IntVar::Big(a) => match other {
                IntVar::Small(b) => match a.to_isize() {
                    Some(i) => i.cmp(b),
                    None => match a.sign() {
                        Sign::Minus => Ordering::Less,
                        Sign::NoSign => {
                            unreachable!("A BigInt with a sign of 0 should be convertible to isize")
                        }
                        Sign::Plus => Ordering::Greater,
                    },
                },
                IntVar::Big(b) => a.cmp(b),
            },
        }
    }
}

impl From<isize> for IntVar {
    fn from(x: isize) -> Self {
        IntVar::Small(x)
    }
}

impl From<IntVar> for BigInt {
    fn from(x: IntVar) -> Self {
        match x {
            IntVar::Small(i) => i.into(),
            IntVar::Big(b) => Rc::try_unwrap(b).unwrap_or_else(|x| (*x).clone()),
        }
    }
}

impl TryFrom<IntVar> for BigUint {
    type Error = IntVar;

    fn try_from(x: IntVar) -> Result<Self, Self::Error> {
        match x {
            IntVar::Small(i) => i.try_into().map_err(|_| IntVar::Small(i)),
            IntVar::Big(b) => match Rc::try_unwrap(b) {
                Result::Ok(x) => BigUint::try_from(x)
                    .map_err(TryFromBigIntError::into_original)
                    .map_err(Into::into),
                Result::Err(e) => e.to_biguint().ok_or_else(|| e.into()),
            },
        }
    }
}

impl FromStr for IntVar {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match isize::from_str(s) {
            Result::Ok(i) => Result::Ok(IntVar::Small(i)),
            Result::Err(_) => {
                Result::Ok(IntVar::Big(Rc::new(BigInt::from_str(s).map_err(|_| ())?)))
            }
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
        self.to_bigint()?.try_into().ok()
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

    fn is_one(&self) -> bool {
        match self {
            IntVar::Small(s) => s.is_one(),
            IntVar::Big(b) => b.is_one(),
        }
    }
}

impl Signed for IntVar {
    fn abs(&self) -> Self {
        match self {
            IntVar::Small(i) => abs(*i).into(),
            IntVar::Big(b) => IntVar::Big(Rc::new(b.abs())),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        match self {
            IntVar::Small(s1) => match other {
                IntVar::Small(s2) => s1.abs_sub(s2).into(),
                IntVar::Big(b2) => match b2.to_isize() {
                    Option::Some(s2) => s1.abs_sub(&s2).into(),
                    Option::None => BigInt::from(*s1).abs_sub(&*b2).into(),
                },
            },
            IntVar::Big(b1) => match other {
                IntVar::Small(s2) => match b1.to_isize() {
                    Option::Some(s1) => s1.abs_sub(&s2).into(),
                    Option::None => b1.abs_sub(&(*s2).into()).into(),
                },
                IntVar::Big(b2) => {
                    if b1 <= b2 {
                        IntVar::zero()
                    } else {
                        b1.abs_sub(b2).into()
                    }
                }
            },
        }
    }

    fn signum(&self) -> Self {
        match self {
            IntVar::Small(i) => i.signum().into(),
            IntVar::Big(b) => match b.sign() {
                Sign::Minus => -IntVar::one(),
                Sign::NoSign => IntVar::zero(),
                Sign::Plus => IntVar::one(),
            },
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

macro_rules! checked_big {
    ($name:ident, $n1:ident, $n2:ident) => {
        match $n2 {
            IntVar::Small(s2) => $n1.as_ref().$name(s2).into(),
            IntVar::Big(b2) => $n1.as_ref().$name(b2.as_ref()).into(),
        }
    };
}

macro_rules! impl_checked {
    ($name:ident, $trait:ty, $checked:ident) => {
        impl $trait for IntVar {
            type Output = IntVar;

            fn $name(self, rhs: Self) -> Self::Output {
                match self {
                    IntVar::Small(s1) => match rhs {
                        IntVar::Small(s2) => match s1.$checked(s2) {
                            Option::Some(val) => IntVar::Small(val),
                            Option::None => BigInt::from(s1).$name(s2).into(),
                        },
                        IntVar::Big(b2) => s1.$name(b2.as_ref()).into(),
                    },
                    IntVar::Big(b1) => checked_big!($name, b1, rhs),
                }
            }
        }
    };

    ($name:ident, $trait:ident, $subtype:ty, $checked:ident) => {
        impl $trait<$subtype> for IntVar {
            type Output = IntVar;

            fn $name(self, rhs: $subtype) -> Self::Output {
                match self {
                    IntVar::Small(s1) => match rhs.try_into() {
                        Result::Ok(s2) => match s1.$checked(s2) {
                            Option::Some(val) => IntVar::Small(val),
                            Option::None => BigInt::from(s1).$name(s2).into(),
                        },
                        Result::Err(_) => BigInt::from(s1).$name(rhs).into(),
                    },
                    IntVar::Big(b1) => b1.as_ref().$name(rhs).into(),
                }
            }
        }
    };

    ($name:ident, $trait:ident, $checked:ident, {$($ty:ty),+ $(,)?}) => {
        $(
            impl_checked!($name, $trait, $ty, $checked);
        )+
    }
}

macro_rules! impl_checked_ref {
    ($name:ident, $trait:ty, $checked:ident) => {
        impl $trait for &IntVar {
            type Output = IntVar;

            fn $name(self, rhs: Self) -> Self::Output {
                match self {
                    IntVar::Small(s1) => match rhs {
                        IntVar::Small(s2) => match s1.$checked(*s2) {
                            Option::Some(val) => IntVar::Small(val),
                            Option::None => BigInt::from(*s1).$name(s2).into(),
                        },
                        IntVar::Big(b2) => s1.$name(b2.as_ref()).into(),
                    },
                    IntVar::Big(b1) => checked_big!($name, b1, rhs),
                }
            }
        }
    };

    ($name:ident, $trait:ident, $subtype:ty, $checked:ident) => {
        impl $trait<$subtype> for &IntVar {
            type Output = IntVar;

            fn $name(self, rhs: $subtype) -> Self::Output {
                match self {
                    IntVar::Small(s1) => match rhs.try_into() {
                        Result::Ok(s2) => match s1.$checked(s2) {
                            Option::Some(val) => IntVar::Small(val),
                            Option::None => BigInt::from(*s1).$name(s2).into(),
                        },
                        Result::Err(_) => BigInt::from(*s1).$name(rhs).into(),
                    },
                    IntVar::Big(b1) => b1.as_ref().$name(rhs).into(),
                }
            }
        }
    };

    ($name:ident, $trait:ident, $checked:ident, {$($ty:ty),+ $(,)?}) => {
        $(
            impl_checked_ref!($name, $trait, $ty, $checked);
        )+
    }
}

macro_rules! impl_assign {
    ($name:ident, $trait:ty, $original:ident) => {
        impl $trait for IntVar {
            fn $name(&mut self, rhs: Self) {
                *self = (self as &Self).$original(&rhs);
            }
        }
    };

    ($name:ident, $trait:ident, $subtype:ty, $original:ident) => {
        impl $trait<$subtype> for IntVar {
            fn $name(&mut self, rhs: $subtype) {
                *self = (self as &Self).$original(rhs);
            }
        }
    };

    ($name:ident, $trait:ident, $original:ident, {$($ty:ty),+ $(,)?}) => {
        $(
            impl_assign!($name, $trait, $ty, $original);
        )+
    }
}

macro_rules! inner_impl {
    ($name:ident, $trait:ty) => {
        fn $name(self, rhs: Self) -> Self::Output {
            match self {
                IntVar::Small(s1) => match rhs {
                    IntVar::Small(s2) => IntVar::Small(s1.$name(s2)),
                    IntVar::Big(b2) => (s1.$name(b2.as_ref())).into(),
                },
                IntVar::Big(b1) => checked_big!($name, b1, rhs),
            }
        }
    };
}

macro_rules! impl_nonzero {
    ($name:ident, $trait:ty, $assign:ident, $assign_tr:ty) => {
        impl $trait for IntVar {
            type Output = IntVar;

            inner_impl!($name, $trait);
        }

        impl $trait for &IntVar {
            type Output = IntVar;

            inner_impl!($name, $trait);
        }

        impl_assign!($assign, $assign_tr, $name);
    };
}

macro_rules! impl_op {
    ($name:ident, $trait:ident, $checked:ident, $assign:ident, $assign_tr:ident) => {
        impl_checked!($name, $trait, $checked);
        impl_checked_ref!($name, $trait, $checked);
        impl_assign!($assign, $assign_tr, $name);

        impl_checked!($name, $trait, $checked, {
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
        });
        impl_checked_ref!($name, $trait, $checked, {
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
        });
        impl_assign!($assign, $assign_tr, $name, {
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
        });
    };
}

impl_op!(add, Add, checked_add, add_assign, AddAssign);
impl_op!(sub, Sub, checked_sub, sub_assign, SubAssign);
impl_op!(mul, Mul, checked_mul, mul_assign, MulAssign);
impl_nonzero!(div, Div, div_assign, DivAssign);
impl_nonzero!(rem, Rem, rem_assign, RemAssign);

impl Neg for IntVar {
    type Output = Self;

    fn neg(self) -> Self::Output {
        (&self).neg()
    }
}

impl Neg for &IntVar {
    type Output = IntVar;

    fn neg(self) -> Self::Output {
        match self {
            IntVar::Small(i) => (-i).into(),
            IntVar::Big(b) => (-b.as_ref()).into(),
        }
    }
}

impl Pow<Self> for IntVar {
    type Output = Self;

    fn pow(self, rhs: Self) -> Self::Output {
        Pow::pow(
            BigInt::from(self),
            BigUint::try_from(rhs).expect("Cannot 'pow' with negative number"),
        )
        .into()
    }
}

impl Shl<usize> for IntVar {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        (BigInt::from(self) << rhs).into()
    }
}

impl Shl<usize> for &IntVar {
    type Output = IntVar;

    fn shl(self, rhs: usize) -> Self::Output {
        match self {
            IntVar::Small(i) => (BigInt::from(*i) << rhs).into(),
            IntVar::Big(b) => (b.as_ref() << rhs).into(),
        }
    }
}

impl Shr<usize> for IntVar {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        (BigInt::from(self) >> rhs).into()
    }
}

impl Shr<usize> for &IntVar {
    type Output = IntVar;

    fn shr(self, rhs: usize) -> Self::Output {
        match self {
            IntVar::Small(i) => (BigInt::from(*i) >> rhs).into(),
            IntVar::Big(b) => (b.as_ref() >> rhs).into(),
        }
    }
}

macro_rules! impl_bit {
    ($trait_name:ty, $fn_name:ident, $assign_trait:ty, $assign_fn:ident) => {
        impl $trait_name for IntVar {
            type Output = Self;

            fn $fn_name(self, rhs: Self) -> Self::Output {
                match self {
                    IntVar::Small(s1) => match rhs {
                        IntVar::Small(s2) => s1.$fn_name(s2).into(),
                        IntVar::Big(b2) => b2.as_ref().$fn_name(&s1.into()).into(),
                    },
                    IntVar::Big(b1) => match rhs {
                        IntVar::Small(s2) => b1.as_ref().$fn_name(&s2.into()).into(),
                        IntVar::Big(b2) => b1.as_ref().$fn_name(&*b2).into(),
                    },
                }
            }
        }

        impl $trait_name for &IntVar {
            type Output = IntVar;

            fn $fn_name(self, rhs: Self) -> Self::Output {
                match self {
                    IntVar::Small(s1) => match rhs {
                        IntVar::Small(s2) => s1.$fn_name(s2).into(),
                        IntVar::Big(b2) => b2.as_ref().$fn_name(&(*s1).into()).into(),
                    },
                    IntVar::Big(b1) => match rhs {
                        IntVar::Small(s2) => b1.as_ref().$fn_name(&(*s2).into()).into(),
                        IntVar::Big(b2) => b1.as_ref().$fn_name(b2.as_ref()).into(),
                    },
                }
            }
        }

        impl $assign_trait for IntVar {
            fn $assign_fn(&mut self, rhs: Self) {
                *self = (self as &Self).$fn_name(&rhs)
            }
        }
    };
}

impl_bit!(BitAnd, bitand, BitAndAssign, bitand_assign);
impl_bit!(BitOr, bitor, BitOrAssign, bitor_assign);
impl_bit!(BitXor, bitxor, BitXorAssign, bitxor_assign);

impl Not for IntVar {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            IntVar::Small(i) => (!i).into(),
            IntVar::Big(b) => (!&*b).into(),
        }
    }
}

macro_rules! impl_display {
    ( $($val:ident),* ) => {
        $(
            impl $val for IntVar {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    match self {
                        IntVar::Small(i) => $val::fmt(i, f),
                        IntVar::Big(b) => $val::fmt(b.as_ref(), f),
                    }
                }
            }
        )+
    };
}

impl_display! {Binary, Display, LowerHex, Octal, UpperHex}

macro_rules! integer_fn {
    ($name:ident) => {
        fn $name(&self, other: &Self) -> Self {
            match self {
                IntVar::Small(s1) => match other {
                    IntVar::Small(s2) => s1.$name(s2).into(),
                    IntVar::Big(b2) => BigInt::from(*s1).$name(&b2).into(),
                },
                IntVar::Big(b1) => match other {
                    IntVar::Small(s2) => b1.$name(&(*s2).into()).into(),
                    IntVar::Big(b2) => b1.$name(&b2).into(),
                },
            }
        }
    };
}

impl Integer for IntVar {
    integer_fn!(div_floor);
    integer_fn!(mod_floor);
    integer_fn!(gcd);
    integer_fn!(lcm);

    fn divides(&self, other: &Self) -> bool {
        self.is_multiple_of(other)
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        (self % other).is_zero()
    }

    fn is_even(&self) -> bool {
        match self {
            IntVar::Small(s) => s.is_even(),
            IntVar::Big(b) => b.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        match self {
            IntVar::Small(s) => s.is_odd(),
            IntVar::Big(b) => b.is_odd(),
        }
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        match self {
            IntVar::Small(s1) => match other {
                IntVar::Small(s2) => map_into(s1.div_rem(s2)),
                IntVar::Big(b2) => map_into(BigInt::from(*s1).div_rem(&b2)),
            },
            IntVar::Big(b1) => match other {
                IntVar::Small(s2) => map_into(b1.div_rem(&(*s2).into())),
                IntVar::Big(b2) => map_into(b1.div_rem(&b2)),
            },
        }
    }
}

fn map_into<X, Y: Into<X>>((a, b): (Y, Y)) -> (X, X) {
    (a.into(), b.into())
}
