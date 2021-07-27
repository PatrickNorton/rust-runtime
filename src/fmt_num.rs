use crate::from_bool::FromBool;
use num::bigint::Sign;
use num::traits::Pow;
use num::{BigInt, BigRational, BigUint, Integer, One, Signed, ToPrimitive, Zero};
use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::f64::consts::LOG10_2;
use std::fmt::{Display, Formatter, LowerExp, UpperExp, Write};

#[derive(Debug, Eq, PartialEq)]
struct FmtDecimal {
    value: BigInt,
    scale: i64,
}

#[derive(Debug, Eq, PartialEq)]
struct FmtDecimalRef<'a> {
    value: &'a BigUint,
    scale: i64,
}

const LONG_TEN_POWERS_TABLE: [u64; 20] = [
    1,                    // 0 / 10^0
    10,                   // 1 / 10^1
    100,                  // 2 / 10^2
    1000,                 // 3 / 10^3
    10000,                // 4 / 10^4
    100000,               // 5 / 10^5
    1000000,              // 6 / 10^6
    10000000,             // 7 / 10^7
    100000000,            // 8 / 10^8
    1000000000,           // 9 / 10^9
    10000000000,          // 10 / 10^10
    100000000000,         // 11 / 10^11
    1000000000000,        // 12 / 10^12
    10000000000000,       // 13 / 10^13
    100000000000000,      // 14 / 10^14
    1000000000000000,     // 15 / 10^15
    10000000000000000,    // 16 / 10^16
    100000000000000000,   // 17 / 10^17
    1000000000000000000,  // 18 / 10^18
    10000000000000000000, // 19 / 10^19
];

const BIG_TEN_POWERS_TABLE_INITLEN: usize = 20;
const BIG_TEN_POWERS_TABLE_MAX: usize = 16 * BIG_TEN_POWERS_TABLE_INITLEN;
static BIG_TEN_POWERS_TABLE: Lazy<[BigInt; BIG_TEN_POWERS_TABLE_INITLEN]> = Lazy::new(|| {
    [
        BigInt::one(),
        BigInt::from(10u64),
        BigInt::from(100u64),
        BigInt::from(1000u64),
        BigInt::from(10000u64),
        BigInt::from(100000u64),
        BigInt::from(1000000u64),
        BigInt::from(10000000u64),
        BigInt::from(100000000u64),
        BigInt::from(1000000000u64),
        BigInt::from(10000000000u64),
        BigInt::from(100000000000u64),
        BigInt::from(1000000000000u64),
        BigInt::from(10000000000000u64),
        BigInt::from(100000000000000u64),
        BigInt::from(1000000000000000u64),
        BigInt::from(10000000000000000u64),
        BigInt::from(100000000000000000u64),
        BigInt::from(1000000000000000000u64),
        BigInt::from(10000000000000000000u64),
    ]
});

pub static BIG_U_TEN: Lazy<BigUint> = Lazy::new(|| BigUint::from(10u32));
pub static BIG_TEN: Lazy<BigInt> = Lazy::new(|| BigInt::from(10u32));

pub fn format_rational_unsigned(value: BigRational, precision: u32) -> String {
    let (numer, denom) = value.into();
    let fmt_dec = FmtDecimal::from_ratio(numer, denom, precision as u64);
    format!("{:.*}", precision as usize, fmt_dec.into_abs())
}

pub fn format_u_exp(value: BigRational, precision: u32) -> String {
    format!(
        "{:.*e}",
        precision as usize,
        get_dec(value, precision).into_abs()
    )
}

pub fn format_upper_u_exp(value: BigRational, precision: u32) -> String {
    format!(
        "{:.*E}",
        precision as usize,
        get_dec(value, precision).into_abs()
    )
}

pub fn format_int_exp(value: &BigUint, precision: u32) -> String {
    format!("{:.*e}", precision as usize, FmtDecimalRef::new(value, 0))
}

pub fn format_int_upper_exp(value: &BigUint, precision: u32) -> String {
    format!("{:.*e}", precision as usize, FmtDecimalRef::new(value, 0))
}

fn get_dec(value: BigRational, precision: u32) -> FmtDecimal {
    let (numer, denom) = value.into();
    // If |value| < 1, the precision we pass won't be enough for it to print
    // the requisite number of digits. As such, we need to pre-calculate the
    // shift (the number after the 'e'), and add that to the requisite
    // precision.
    let precision = if numer.magnitude() < denom.magnitude() {
        // The shift over should be <= to the number of the digits of the remainder
        let digits = digit_count(&denom);
        precision as u64 + digits
    } else {
        precision as u64
    };
    FmtDecimal::from_ratio(numer, denom, precision)
}

impl<'a> FmtDecimalRef<'a> {
    fn new(int_val: &'a BigUint, scale: i64) -> FmtDecimalRef<'a> {
        FmtDecimalRef {
            value: int_val,
            scale,
        }
    }
}

impl FmtDecimal {
    pub fn from_ratio(numer: BigInt, denom: BigInt, decimal_digs: u64) -> FmtDecimal {
        Self::divide_integer(numer, denom, decimal_digs.try_into().unwrap())
    }

    fn new(int_val: BigInt, scale: i64) -> FmtDecimal {
        FmtDecimal {
            value: int_val,
            scale,
        }
    }

    pub fn into_abs(self) -> FmtDecimal {
        let (_, value) = self.value.into_parts();
        FmtDecimal {
            value: BigInt::from(value),
            scale: self.scale,
        }
    }

    fn divide_integer(dividend: BigInt, divisor: BigInt, scale: i64) -> FmtDecimal {
        if dividend.is_zero() {
            FmtDecimal::new(BigInt::zero(), scale)
        } else if divisor.is_one() {
            FmtDecimal::new(dividend, 0)
        } else if dividend == divisor {
            FmtDecimal::new(Self::big_ten_to_the(scale).into_owned(), scale)
        } else if scale > 0 {
            let scaled_dividend = Self::big_multiply_power_ten(dividend, scale);
            Self::divide_and_round(scaled_dividend, divisor, scale, scale)
        } else {
            let scaled_divisor = Self::big_multiply_power_ten(divisor, -scale);
            Self::divide_and_round(dividend, scaled_divisor, scale, scale)
        }
    }

    fn big_multiply_power_ten(value: BigInt, n: i64) -> BigInt {
        if n <= 0 {
            value
        } else if (n as usize) < LONG_TEN_POWERS_TABLE.len() {
            value * LONG_TEN_POWERS_TABLE[n as usize]
        } else {
            value * &*Self::big_ten_to_the(n)
        }
    }

    fn divide_and_round(
        dividend: BigInt,
        divisor: BigInt,
        scale: i64,
        preferred_scale: i64,
    ) -> FmtDecimal {
        let (mq, mr) = dividend.div_rem(&divisor);
        let is_remainder_zero = mr.is_zero();
        let q_sign = mq.sign();
        if !is_remainder_zero {
            if Self::needs_increment(&divisor, q_sign, &mr) {
                FmtDecimal::new(mq + 1, scale)
            } else {
                FmtDecimal::new(mq, scale)
            }
        } else if preferred_scale != scale {
            Self::create_and_strip_zeros_to_match_scale(mq, scale, preferred_scale)
        } else {
            FmtDecimal::new(mq, scale)
        }
    }

    fn needs_increment(divisor: &BigInt, q_sign: Sign, mr: &BigInt) -> bool {
        assert!(!mr.is_zero());
        let cmp_frac_half = compare_half(mr, divisor);
        Self::common_need_increment(q_sign, cmp_frac_half)
    }

    fn common_need_increment(_q_sign: Sign, cmp_frac_half: Ordering) -> bool {
        // We round half up here
        match cmp_frac_half {
            Ordering::Less => false,
            Ordering::Greater => true,
            Ordering::Equal => true,
        }
    }

    fn create_and_strip_zeros_to_match_scale(
        mut int_val: BigInt,
        mut scale: i64,
        preferred_scale: i64,
    ) -> FmtDecimal {
        while int_val.magnitude() > &*BIG_U_TEN && scale > preferred_scale {
            if int_val.is_odd() {
                break; // odd number cannot end in 0
            }
            let (quot, rem) = int_val.div_rem(&*BIG_TEN);
            if !rem.is_zero() {
                break; // non-0 remainder
            }
            int_val = quot;
            scale -= 1;
        }
        FmtDecimal::new(int_val, scale)
    }

    fn big_ten_to_the(n: i64) -> Cow<'static, BigInt> {
        if n < 0 {
            Cow::Owned(BigInt::zero())
        } else if n < BIG_TEN_POWERS_TABLE_MAX as i64 {
            let powers = &*BIG_TEN_POWERS_TABLE;
            if n < powers.len() as i64 {
                Cow::Borrowed(&powers[n as usize])
            } else {
                // TODO? expand_big_integer_ten_powers(n)
                Cow::Owned(Pow::pow(BigInt::from(10), n as u64))
            }
        } else {
            Cow::Owned(Pow::pow(BigInt::from(10), n as u64))
        }
    }
}

impl Display for FmtDecimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_display(&self.value, self.scale, f)
    }
}

impl LowerExp for FmtDecimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_exp(&self.value, self.scale, f, 'e')
    }
}

impl UpperExp for FmtDecimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_exp(&self.value, self.scale, f, 'E')
    }
}

impl Display for FmtDecimalRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_display_abs(self.value, self.scale, f)
    }
}

impl LowerExp for FmtDecimalRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_exp_abs(self.value, self.scale, f, 'e')
    }
}

impl UpperExp for FmtDecimalRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_exp_abs(self.value, self.scale, f, 'E')
    }
}

fn fmt_display(value: &BigInt, scale: i64, f: &mut Formatter<'_>) -> std::fmt::Result {
    if scale == 0 {
        return value.fmt(f);
    }
    if value.is_negative() {
        f.write_char('-')?;
    }
    fmt_display_abs(value.magnitude(), scale, f)
}

fn fmt_display_abs(value: &BigUint, scale: i64, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut digits = get_digits(value);
    let digit_len = digits.len();
    let pad = scale - digit_len as i64;
    if pad >= 0 {
        let pad = pad as usize;
        match f.precision() {
            Option::None => {
                f.write_char('0')?;
                f.write_char('.')?;
                for _ in 0..pad {
                    f.write_char('0')?;
                }
                for ch in digits {
                    f.write_char(ch)?;
                }
            }
            Option::Some(prec) => {
                if prec < pad {
                    f.write_char('0')?;
                    f.write_char('.')?;
                    for _ in 0..prec {
                        f.write_char('0')?;
                    }
                } else if prec - pad >= digit_len {
                    f.write_char('0')?;
                    f.write_char('.')?;
                    for _ in 0..pad {
                        f.write_char('0')?;
                    }
                    for ch in digits {
                        f.write_char(ch)?;
                    }
                    for _ in 0..prec - pad - digit_len {
                        f.write_char('0')?;
                    }
                } else {
                    let carry = round_chars(&mut digits, prec - pad);
                    f.write_char(if carry && pad == 0 { '1' } else { '0' })?;
                    f.write_char('.')?;
                    for _ in 0..pad.saturating_sub(1) {
                        f.write_char('0')?;
                    }
                    if pad > 0 {
                        f.write_char(if carry { '1' } else { '0' })?;
                    }
                    for ch in &digits[..prec - pad] {
                        f.write_char(*ch)?;
                    }
                }
            }
        }
    } else {
        let pad = (-pad) as usize;
        match f.precision() {
            Option::None => {
                for ch in &digits[..pad] {
                    f.write_char(*ch)?;
                }
                f.write_char('.')?;
                for ch in &digits[pad..] {
                    f.write_char(*ch)?;
                }
            }
            Option::Some(x) => {
                if x + pad >= digit_len {
                    for ch in &digits[..pad] {
                        f.write_char(*ch)?;
                    }
                    f.write_char('.')?;
                    for ch in &digits[pad..] {
                        f.write_char(*ch)?;
                    }
                    for _ in 0..x + pad - digit_len {
                        f.write_char('0')?;
                    }
                } else {
                    let carry = round_chars(&mut digits, x + pad);
                    if carry {
                        f.write_char('1')?;
                    }
                    for ch in &digits[..pad] {
                        f.write_char(*ch)?;
                    }
                    f.write_char('.')?;
                    for ch in &digits[pad..x + pad] {
                        f.write_char(*ch)?;
                    }
                }
            }
        }
    }
    Result::Ok(())
}

fn fmt_exp(value: &BigInt, scale: i64, f: &mut Formatter<'_>, e: char) -> std::fmt::Result {
    if scale == 0 {
        return value.fmt(f);
    }
    if value.is_negative() {
        f.write_char('-')?;
    }
    fmt_exp_abs(value.magnitude(), scale, f, e)
}

fn fmt_exp_abs(value: &BigUint, scale: i64, f: &mut Formatter<'_>, e: char) -> std::fmt::Result {
    let precision = f.precision();
    let mut digits = get_digits(value);
    let sub_one;
    if digits.len() > 1 {
        match precision {
            Option::Some(x) => {
                if x + 1 >= digits.len() {
                    f.write_char(digits[0])?;
                    f.write_char('.')?;
                    for ch in &digits[1..] {
                        f.write_char(*ch)?;
                    }
                    for _ in 0..x + 1 - digits.len() {
                        f.write_char('0')?;
                    }
                    sub_one = true;
                } else {
                    let carry = round_chars(&mut digits, x + 1);
                    f.write_char(if carry { '1' } else { digits[0] })?;
                    f.write_char('.')?;
                    let c = usize::from_bool(!carry);
                    for ch in &digits[c..x + c] {
                        f.write_char(*ch)?;
                    }
                    sub_one = !carry;
                }
            }
            Option::None => {
                f.write_char(digits[0])?;
                f.write_char('.')?;
                for ch in &digits[1..] {
                    f.write_char(*ch)?;
                }
                sub_one = true;
            }
        }
    } else {
        f.write_char(digits[0])?;
        if let Option::Some(prec) = precision {
            f.write_char('.')?;
            for _ in 0..prec {
                f.write_char('0')?;
            }
        }
        sub_one = true;
    }
    let adjusted = -scale + (digits.len() as i64) - i64::from_bool(sub_one);
    f.write_char(e)?;
    write!(f, "{:+03}", adjusted)?;
    Result::Ok(())
}

fn get_digits(value: &BigUint) -> Vec<char> {
    value.to_string().chars().collect()
}

/// Computes `a.cmp(b/2)`.
///
/// This is faster than the standard computation, as this avoids any
/// intermediate allocation.
///
/// # Examples
///
/// ```
/// use num::BigInt;
/// use std::cmp::Ordering;
///
/// let a = BigInt::from(10);
/// let b = BigInt::from(15);
/// let c = BigInt::from(20);
/// let d = BigInt::from(30);
/// assert_eq!(compare_half(&a, &b), Ordering::Greater);
/// assert_eq!(compare_half(&a, &c), Ordering::Equal);
/// assert_eq!(compare_half(&a, &d), Ordering::Less);
/// ```
fn compare_half(a: &BigInt, b: &BigInt) -> Ordering {
    // TODO: Replace
    // let a_val = a.iter_u32_digits_be();
    // let mut b_val = b.iter_u32_digits_be();
    let a_val = a.to_u32_digits().1.into_iter().rev();
    let mut b_val = b.to_u32_digits().1.into_iter().rev();
    let a_len = a_val.len();
    let b_len = b_val.len();
    if a_len == 0 {
        return a_len.cmp(&b_len);
    } else if a_len > b_len {
        return Ordering::Greater;
    } else if a_len < b_len - 1 {
        return Ordering::Less;
    }
    let mut carry = 0u32;
    // Only 2 cases left: a_len == b_len or a_len == b_len - 1
    if a_len != b_len {
        // a_len == b_len - 1
        if b_val.next().unwrap() == 1 {
            carry = 0x8000_0000;
        } else {
            return Ordering::Less;
        }
    }
    for (av, bv) in a_val.zip(b_val) {
        let hb = (bv >> 1) + carry;
        if av != hb {
            return av.cmp(&hb);
        }
        carry = (bv & 1) << (u32::BITS - 1); // carry will be either 0x80000000 or 0
    }
    if carry == 0 {
        Ordering::Equal
    } else {
        Ordering::Less
    }
}

/// Computes `a.cmp(b/10)`.
///
/// This is faster than the standard computation, as this avoids any
/// intermediate allocation.
///
/// # Examples
///
/// ```
/// use num::BigInt;
/// use std::cmp::Ordering;
///
/// let a = BigUint::from(10);
/// let b = BigUint::from(50);
/// let c = BigUint::from(100);
/// let d = BigUint::from(150);
/// assert_eq!(compare_tenth(&a, &b), Ordering::Greater);
/// assert_eq!(compare_tenth(&a, &c), Ordering::Equal);
/// assert_eq!(compare_tenth(&a, &d), Ordering::Less);
/// ```
fn compare_tenth(a: &BigUint, b: &BigUint) -> Ordering {
    // TODO: Replace
    // let a_val = a.iter_u32_digits_be();
    // let mut b_val = b.iter_u32_digits_be();
    let a_val = a.to_u32_digits().into_iter().rev();
    let mut b_val = b.to_u32_digits().into_iter().rev();
    let a_len = a_val.len();
    let b_len = b_val.len();
    if a_len == 0 {
        return a_len.cmp(&b_len);
    } else if a_len > b_len {
        return Ordering::Greater;
    } else if a_len < b_len - 1 {
        return Ordering::Less;
    }
    let mut carry = 0;
    // Only 2 cases left: a_len == b_len or a_len == b_len - 1
    if a_len != b_len {
        // a_len = b_len - 1
        let next = b_val.next().unwrap();
        if next < 10 {
            carry = next % 10;
        } else {
            return Ordering::Less;
        }
    }
    for (av, bv) in a_val.zip(b_val) {
        // Lots of scary integer-size manipulation here, but nothing should
        // overflow (`hb as u32` should be fine because big is < 10 *
        // u32::MAX, thus hb is < u32::MAX, and `rem as u32` is fine because
        // rem is < 10).
        // In essence, what we're doing is converting from 32 to 64 bits,
        // tacking on the remainder at the front, and then dividing and
        // converting back.
        let big = bv as u64 | ((carry as u64) << u32::BITS as u64);
        let (hb, rem) = big.div_rem(&10);
        if av != hb as u32 {
            return av.cmp(&(hb as u32));
        }
        carry = rem as u32;
    }
    if carry == 0 {
        Ordering::Equal
    } else {
        Ordering::Less
    }
}

/// Computes `a.cmp(b*10)`.
///
/// This is faster than the standard computation, as this avoids any
/// intermediate allocation.
///
/// # Examples
///
/// ```
/// use num::BigUint;
/// use std::cmp::Ordering;
///
/// let a = BigUint::from(10);
/// let b = BigUint::from(50);
/// let c = BigUint::from(100);
/// let d = BigUint::from(150);
/// assert_eq!(compare_ten(&b, &a), Ordering::Less);
/// assert_eq!(compare_ten(&c, &a), Ordering::Equal);
/// assert_eq!(compare_ten(&d, &a), Ordering::Greater);
/// ```
fn compare_ten(a: &BigUint, b: &BigUint) -> Ordering {
    compare_tenth(b, a).reverse()
}

fn digit_count(x: &BigInt) -> u64 {
    // Fast paths for small numbers
    if x.is_zero() {
        1
    } else if let Option::Some(x) = x.magnitude().to_u64() {
        match LONG_TEN_POWERS_TABLE.binary_search(&x) {
            Result::Ok(x) => (x + 1) as u64,
            Result::Err(x) => x as u64,
        }
    } else {
        big_digit_count(x.magnitude())
    }
}

fn big_digit_count(x: &BigUint) -> u64 {
    let mut approx = (x.bits() as f64 * LOG10_2) as u64;
    let mut approx_pow = Pow::pow(&*BIG_U_TEN, approx);
    // There are loops in this next section, but almost all of the time,
    // approx_pow() will be accurate, and so we can avoid loops.
    match x.cmp(&approx_pow) {
        // Don't get confused by the compare_tenth() calls here: it's trying to
        // prevent unnecessary multiplication/division wherever possible (in
        // almost all cases, we can avoid any at all)
        Ordering::Less => {
            approx -= 1;
            while compare_tenth(x, &approx_pow).is_lt() {
                approx -= 1;
                approx_pow /= 10u32;
            }
        }
        Ordering::Equal => {}
        Ordering::Greater => loop {
            match compare_ten(x, &approx_pow) {
                Ordering::Less => break,
                Ordering::Equal => {
                    approx += 1;
                    break;
                }
                Ordering::Greater => {
                    approx += 1;
                    approx_pow *= 10u32;
                }
            }
        },
    }
    approx + 1
}

fn char_rounds_up(ch: char) -> bool {
    match ch {
        '0'..='4' => false,
        '5'..='9' => true,
        _ => unreachable!("{} is not a decimal digit", ch),
    }
}

fn increment_digit(ch: &mut char) -> bool {
    *ch = match *ch {
        '0' => '1',
        '1' => '2',
        '2' => '3',
        '3' => '4',
        '5' => '6',
        '6' => '7',
        '7' => '8',
        '8' => '9',
        '9' => '0',
        _ => unreachable!("{} is not a decimal digit", ch),
    };
    *ch == '0'
}

fn round_chars(chars: &mut [char], last: usize) -> bool {
    if last >= chars.len() {
        return false;
    }
    let mut carry = char_rounds_up(chars[last]);
    for digit in chars[..last].iter_mut().rev() {
        if !carry {
            return false;
        } else {
            carry = increment_digit(digit);
        }
    }
    carry
}

#[cfg(test)]
mod test {
    use crate::fmt_num::{
        big_digit_count, compare_half, compare_ten, compare_tenth, digit_count, FmtDecimal,
        BIG_TEN, BIG_U_TEN,
    };
    use num::traits::Pow;
    use num::{BigInt, BigUint, One, Zero};
    use std::cmp::Ordering;

    #[test]
    fn dig_count() {
        assert_eq!(digit_count(&BigInt::zero()), 1);
        assert_eq!(digit_count(&BigInt::from(100)), 3);
        assert_eq!(digit_count(&BigInt::from(123)), 3);
        assert_eq!(digit_count(&Pow::pow(&*BIG_TEN, 75u32)), 76);
    }

    #[test]
    fn comp_half() {
        let a = BigInt::from(10);
        let b = BigInt::from(15);
        let c = BigInt::from(20);
        let d = BigInt::from(30);
        assert_eq!(compare_half(&a, &b), Ordering::Greater);
        assert_eq!(compare_half(&a, &c), Ordering::Equal);
        assert_eq!(compare_half(&a, &d), Ordering::Less);
    }

    #[test]
    fn comp_tenth() {
        let a = BigUint::from(10u32);
        let b = BigUint::from(50u32);
        let c = BigUint::from(100u32);
        let d = BigUint::from(150u32);
        assert_eq!(compare_tenth(&a, &b), Ordering::Greater);
        assert_eq!(compare_tenth(&a, &c), Ordering::Equal);
        assert_eq!(compare_tenth(&a, &d), Ordering::Less);
    }

    #[test]
    fn comp_ten() {
        let a = BigUint::from(10u32);
        let b = BigUint::from(50u32);
        let c = BigUint::from(100u32);
        let d = BigUint::from(150u32);
        assert_eq!(compare_ten(&b, &a), Ordering::Less);
        assert_eq!(compare_ten(&c, &a), Ordering::Equal);
        assert_eq!(compare_ten(&d, &a), Ordering::Greater);
    }

    #[test]
    fn create_decimal() {
        let e1 = FmtDecimal::new(BigInt::from(333333), 6);
        let a1 = FmtDecimal::from_ratio(BigInt::one(), BigInt::from(3), 6);
        assert_eq!(e1, a1);

        let e2 = FmtDecimal::new(BigInt::from(666667), 6);
        let a2 = FmtDecimal::from_ratio(BigInt::from(2), BigInt::from(3), 6);
        assert_eq!(e2, a2);
    }

    #[test]
    fn fixed_fmt() {
        let d1 = FmtDecimal::new(BigInt::from(333333), 6);
        assert_eq!(&format!("{}", d1), "0.333333");
        assert_eq!(&format!("{:.5}", d1), "0.33333");
        assert_eq!(&format!("{:.7}", d1), "0.3333330");
    }

    #[test]
    fn fixed_round() {
        let d1 = FmtDecimal::new(BigInt::from(666666), 6);
        let d2 = FmtDecimal::new(BigInt::from(999999), 6);
        let d3 = FmtDecimal::new(BigInt::from(999999), 5);
        assert_eq!(&format!("{:.5}", d1), "0.66667");
        assert_eq!(&format!("{:.5}", d2), "1.00000");
        assert_eq!(&format!("{:.4}", d3), "10.0000");
    }

    #[test]
    fn exp_fmt() {
        let d1 = FmtDecimal::new(BigInt::from(333333), 6);
        assert_eq!(&format!("{:e}", d1), "3.33333e-01");
        assert_eq!(&format!("{:.4e}", d1), "3.3333e-01");
        assert_eq!(&format!("{:.6e}", d1), "3.333330e-01");
    }

    #[test]
    fn upper_exp_fmt() {
        let d1 = FmtDecimal::new(BigInt::from(333333), 6);
        assert_eq!(&format!("{:E}", d1), "3.33333E-01");
        assert_eq!(&format!("{:.4E}", d1), "3.3333E-01");
        assert_eq!(&format!("{:.6E}", d1), "3.333330E-01");
    }

    #[test]
    fn exp_carry() {
        let d1 = FmtDecimal::new(BigInt::from(666666), 6);
        let d2 = FmtDecimal::new(BigInt::from(999999), 6);
        let d3 = FmtDecimal::new(BigInt::from(999999), 5);
        assert_eq!(&format!("{:.4e}", d1), "6.6667e-01");
        assert_eq!(&format!("{:.4e}", d2), "1.0000e+00");
        assert_eq!(&format!("{:.4e}", d3), "1.0000e+01");
    }

    #[test]
    fn big_u_digits() {
        assert_eq!(big_digit_count(&BIG_U_TEN), 2);
        let mut d1 = Pow::pow(&*BIG_U_TEN, 99u32);
        assert_eq!(big_digit_count(&d1), 100);
        d1 -= 1u32;
        assert_eq!(big_digit_count(&d1), 99);
        d1 += 2u32;
        assert_eq!(big_digit_count(&d1), 100);
    }
}
