use num::bigint::Sign;
use num::{BigInt, BigRational, One, Signed, Zero};
use std::iter::{Product, Sum};
use std::ops::{
    Add, AddAssign, Deref, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};
use std::rc::Rc;

#[derive(Clone, Debug, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub struct RationalVar {
    value: Rc<BigRational>,
}

impl RationalVar {
    pub fn new(value: BigRational) -> RationalVar {
        RationalVar {
            value: Rc::new(value),
        }
    }

    pub fn into_inner(self) -> BigRational {
        Rc::try_unwrap(self.value).unwrap_or_else(|x| (*x).clone())
    }

    pub fn from_integer(x: BigInt) -> RationalVar {
        Self::new(BigRational::from_integer(x))
    }

    pub fn sign(&self) -> Sign {
        if self.is_zero() {
            Sign::NoSign
        } else if self.is_positive() {
            Sign::Plus
        } else {
            // self.is_negative()
            Sign::Minus
        }
    }
}

impl Deref for RationalVar {
    type Target = BigRational;

    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}

macro_rules! impl_op {
    ($trait:ty, $fn_name: ident, $op:tt) => {
        impl $trait for RationalVar {
            type Output = Self;

            fn $fn_name(self, rhs: Self) -> Self::Output {
                Self::new(&*self $op &*rhs)
            }
        }
    };
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);
impl_op!(Rem, rem, %);

macro_rules! impl_iop {
    ($trait:ty, $fn_name: ident, $op:tt) => {
        impl $trait for RationalVar {
            fn $fn_name(&mut self, rhs: Self) {
                *self = self.clone() $op rhs;
            }
        }
    };
}

impl_iop!(AddAssign, add_assign, +);
impl_iop!(SubAssign, sub_assign, -);
impl_iop!(MulAssign, mul_assign, *);
impl_iop!(DivAssign, div_assign, /);
impl_iop!(RemAssign, rem_assign, %);

impl Neg for RationalVar {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-(*self).clone())
    }
}

impl Zero for RationalVar {
    fn zero() -> Self {
        RationalVar::new(Zero::zero())
    }

    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

impl One for RationalVar {
    fn one() -> Self {
        RationalVar::new(One::one())
    }
    
    fn is_one(&self) -> bool {
        self.value.is_one()
    }
}

impl From<BigRational> for RationalVar {
    fn from(x: BigRational) -> Self {
        Self::new(x)
    }
}

impl Sum for RationalVar {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(BigRational::zero(), |sum, num| sum + &*num)
            .into()
    }
}

impl Product for RationalVar {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(BigRational::one(), |sum, num| sum * &*num).into()
    }
}
