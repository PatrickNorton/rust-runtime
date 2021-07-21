use crate::custom_var::CustomVar;
use crate::first_n;
use crate::int_tools::bytes_index;
use crate::int_var::IntVar;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::{MaybeString, OwnedStringVar, StringVar};
use crate::variable::{FnResult, InnerVar, Variable};
use ascii::{AsciiChar, AsciiStr};
use num::pow::Pow;
use num::{bigint, BigInt, BigRational, BigUint, One, Signed, ToPrimitive, Zero};
use once_cell::sync::Lazy;
use std::fmt::{Display, Formatter, Write};
use std::rc::Rc;

const DEFAULT_FLOAT_DECIMALS: usize = 6;

static BIG_U_TEN: Lazy<BigUint> = Lazy::new(|| BigUint::from(10u32));
static BIG_TEN: Lazy<BigInt> = Lazy::new(|| BigInt::from(10));
static BIG_TEN_RATIONAL: Lazy<BigRational> =
    Lazy::new(|| BigRational::from_integer((*BIG_TEN).clone()));

#[derive(Debug, Default)]
pub struct FormatArgs {
    fill: char,
    align: Align,
    sign: Sign,
    hash: bool,
    zero: bool,
    min_width: u32,
    precision: u32,
    fmt_type: FmtType,
}

pub fn format_internal(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let [arg, formatter] = first_n(args);
    let format = get_formatter(formatter);
    let result = format.format(arg, runtime)?;
    runtime.return_1(result.into())
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Align {
    Left,
    Right,
    AfterSign,
    Center,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Sign {
    Both,
    NegativeOnly,
    LeadingSpace,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum FmtType {
    Binary,
    Character,
    Decimal,
    Octal,
    Hex,
    UpperHex,
    Number,
    Exponent,
    UpperExp,
    Fixed,
    UpperFixed,
    General,
    UpperGeneral,
    Percentage,
    Repr,
    String,
}

impl FormatArgs {
    pub fn parse(bytes: &[u8], index: &mut usize) -> FormatArgs {
        let fill = bytes_index::<char>(bytes, index);
        let align = bytes_index::<u8>(bytes, index);
        let sign = bytes_index::<u8>(bytes, index);
        let hash_zero = bytes_index::<u8>(bytes, index);
        let min_width = bytes_index::<u32>(bytes, index);
        let precision = bytes_index::<u32>(bytes, index);
        let fmt_type = bytes_index::<u8>(bytes, index);
        FormatArgs {
            fill,
            align: Align::from_u8(align),
            sign: Sign::from_u8(sign),
            hash: (hash_zero & 0b01) != 0,
            zero: (hash_zero & 0b10) != 0,
            min_width,
            precision,
            fmt_type: FmtType::from_u8(fmt_type),
        }
    }

    pub fn format(&self, arg: Variable, runtime: &mut Runtime) -> Result<StringVar, ()> {
        match self.fmt_type {
            FmtType::Binary => Result::Ok(self.fmt_binary(arg).into()),
            FmtType::Character => Result::Ok(self.fmt_character(arg).into()),
            FmtType::Decimal => Result::Ok(self.fmt_decimal(arg).into()),
            FmtType::Octal => Result::Ok(self.fmt_octal(arg).into()),
            FmtType::Hex => Result::Ok(self.fmt_hex(arg).into()),
            FmtType::UpperHex => Result::Ok(self.fmt_upper_hex(arg).into()),
            FmtType::Number => Result::Ok(self.fmt_number(arg).into()),
            FmtType::Exponent => Result::Ok(self.fmt_exp(arg).into()),
            FmtType::UpperExp => Result::Ok(self.fmt_upper_exp(arg).into()),
            FmtType::Fixed => Result::Ok(self.fmt_fixed(arg).into()),
            FmtType::UpperFixed => Result::Ok(self.fmt_upper_fixed(arg).into()),
            FmtType::General => Result::Ok(self.fmt_general(arg).into()),
            FmtType::UpperGeneral => Result::Ok(self.fmt_upper_general(arg).into()),
            FmtType::Percentage => Result::Ok(self.fmt_percentage(arg).into()),
            FmtType::Repr => arg.repr(runtime),
            FmtType::String => arg.str(runtime),
        }
    }

    fn is_simple_format(&self) -> bool {
        self.fill == '\0'
            && self.align == Align::Left
            && self.sign == Sign::NegativeOnly
            && !self.hash
            && !self.zero
            && self.min_width == 0
            && self.precision == 0
    }

    fn pad_integer(
        &self,
        mut value: OwnedStringVar,
        sign: bigint::Sign,
        prefix: &str,
    ) -> OwnedStringVar {
        if self.is_simple_format() {
            return value;
        }
        if self.precision != 0 {
            panic!("Precision not allowed in integer format specifier");
        }
        if self.hash {
            value.insert_str(0, prefix);
        }
        let sign_chr = self.sign_char(sign);
        if let Option::Some(sign) = sign_chr {
            value.insert(0, sign);
        }
        if let Option::Some(diff) = value.char_len().checked_sub(self.min_width as usize) {
            if self.zero && self.fill == '\0' {
                let start = if sign_chr.is_some() { 1 } else { 0 }
                    + if self.hash { prefix.len() } else { 0 };
                value.insert_n_chr(start, diff, '0');
            } else {
                let fill_char = if self.fill != '\0' { self.fill } else { ' ' };
                match self.align {
                    Align::Left => value.insert_n_chr(0, diff, fill_char),
                    Align::Right => value.push_n_chr(diff, fill_char),
                    Align::AfterSign => {
                        value.insert_n_chr(if sign_chr.is_some() { 1 } else { 0 }, diff, fill_char)
                    }
                    Align::Center => {
                        let pre_count = diff / 2; // Rounds down
                        let post_count = (diff + 1) / 2; // Rounds up
                        value.insert_n_chr(0, pre_count, fill_char);
                        value.push_n_chr(post_count, fill_char);
                    }
                }
            }
        }
        value
    }

    fn sign_char(&self, sign: bigint::Sign) -> Option<char> {
        match self.sign {
            Sign::Both => match sign {
                bigint::Sign::Minus => Option::Some('-'),
                bigint::Sign::NoSign => Option::Some('+'),
                bigint::Sign::Plus => Option::Some('+'),
            },
            Sign::NegativeOnly => match sign {
                bigint::Sign::Minus => Option::Some('-'),
                bigint::Sign::NoSign => Option::None,
                bigint::Sign::Plus => Option::None,
            },
            Sign::LeadingSpace => match sign {
                bigint::Sign::Minus => Option::Some('-'),
                bigint::Sign::NoSign => Option::Some(' '),
                bigint::Sign::Plus => Option::Some(' '),
            },
        }
    }

    fn fmt_binary(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{:b}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "0b")
    }

    fn fmt_decimal(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "")
    }

    fn fmt_octal(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{:o}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "0o")
    }

    fn fmt_hex(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{:x}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "0x")
    }

    fn fmt_upper_hex(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{:X}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "0X")
    }

    fn fmt_character(&self, var: Variable) -> MaybeString {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        match var {
            // FIXME: This unwrap() should throw an exception instead of panicking
            Variable::Normal(InnerVar::Bigint(i)) => Self::char_from_int(&i).unwrap(),
            Variable::Normal(InnerVar::Bool(b)) => Self::char_from_bool(b),
            Variable::Normal(InnerVar::Char(c)) => c.into(),
            x => panic!(
                "Attempted to turn a variable not a superclass of int ({}) into an int",
                x.get_type().str()
            ),
        }
    }

    fn char_from_int(i: &IntVar) -> Option<MaybeString> {
        i.to_u32().and_then(char::from_u32).map(From::from)
    }

    fn char_from_bool(i: bool) -> MaybeString {
        if i { AsciiChar::SOH } else { AsciiChar::Null }.into()
    }

    fn fmt_number(&self, var: Variable) -> OwnedStringVar {
        self.fmt_decimal(var)
    }

    fn fmt_exp(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        match var {
            Variable::Normal(InnerVar::Bigint(i)) => {
                self.fmt_exp_inner(&BigRational::from(BigInt::from(i)), false)
            }
            Variable::Normal(InnerVar::Bool(b)) => {
                if b {
                    "1.000000e+00".into()
                } else {
                    "0.000000e+00".into()
                }
            }
            Variable::Normal(InnerVar::Decimal(c)) => self.fmt_exp_inner(&*c, false),
            _ => panic!(),
        }
    }

    fn fmt_upper_exp(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        match var {
            Variable::Normal(InnerVar::Bigint(i)) => {
                self.fmt_exp_inner(&BigRational::from(BigInt::from(i)), true)
            }
            Variable::Normal(InnerVar::Bool(b)) => {
                if b {
                    "1.000000E+00".into()
                } else {
                    "0.000000E+00".into()
                }
            }
            Variable::Normal(InnerVar::Decimal(c)) => self.fmt_exp_inner(&*c, true),
            _ => panic!(),
        }
    }

    fn fmt_exp_inner(&self, value: &BigRational, uppercase: bool) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        let abs_value = value.abs();
        let cased_e = if uppercase { 'E' } else { 'e' };
        if value.is_zero() {
            if uppercase {
                "0.000000E+00".into()
            } else {
                "0.000000e+00".into()
            }
        } else if abs_value < *BIG_TEN_RATIONAL && abs_value >= BigRational::one() {
            let mut result = as_decimal(value, DEFAULT_FLOAT_DECIMALS);
            result.push(cased_e);
            result.push_str("+00");
            OwnedStringVar::from_str_checked(result)
        } else if abs_value < BigRational::one() {
            // FIXME: This is *horrifically* slow
            let mut count = 0;
            let mut result = value.clone();
            while result.abs() < BigRational::one() {
                result *= &*BIG_TEN;
                count += 1;
            }
            let result_str = as_decimal(&result, DEFAULT_FLOAT_DECIMALS);
            OwnedStringVar::from_str_checked(format!("{}{}-{}", result_str, cased_e, count))
        } else {
            // FIXME: digit_count still reallocates O(n) times; it would be
            //  nice if there were an algorithm for determining decimal digit
            //  placement without reallocation necessary.
            let (round_num, _) = value.round().into();
            let decimal_digits = digit_count(round_num);
            let result = value / (Pow::pow(&*BIG_TEN, decimal_digits) - 1);
            let result_str = as_decimal(&result, DEFAULT_FLOAT_DECIMALS);
            OwnedStringVar::from_str_checked(format!(
                "{}{}+{}",
                result_str,
                cased_e,
                decimal_digits - 1
            ))
        }
    }

    fn fmt_fixed(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        static ZERO_FIXED: Lazy<&AsciiStr> =
            Lazy::new(|| AsciiStr::from_ascii("0.000000").unwrap());
        static ONE_FIXED: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii("1.000000").unwrap());
        match var {
            Variable::Normal(InnerVar::Bigint(i)) => {
                OwnedStringVar::from_str_checked(format!("{}.000000", i))
            }
            Variable::Normal(InnerVar::Bool(b)) => if b { *ONE_FIXED } else { *ZERO_FIXED }.into(),
            Variable::Normal(InnerVar::Decimal(d)) => {
                // FIXME: Use BigRational better formatting if/when it comes
                //  (see https://github.com/rust-num/num-rational/issues/10)
                OwnedStringVar::from_str_checked(as_decimal(&*d, DEFAULT_FLOAT_DECIMALS))
            }
            _ => panic!(),
        }
    }

    fn fmt_upper_fixed(&self, var: Variable) -> OwnedStringVar {
        let mut format = self.fmt_fixed(var);
        format.make_ascii_uppercase();
        format
    }

    fn fmt_general(&self, var: Variable) -> OwnedStringVar {
        todo!()
    }

    fn fmt_upper_general(&self, var: Variable) -> OwnedStringVar {
        let mut format = self.fmt_general(var);
        format.make_ascii_uppercase();
        format
    }

    fn fmt_percentage(&self, var: Variable) -> OwnedStringVar {
        if !self.is_simple_format() {
            todo!("Non-trivial formatting")
        }
        static ZERO_PCT: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii("00%").unwrap());
        static HUNDRED_PCT: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii("100%").unwrap());
        match var {
            Variable::Normal(InnerVar::Bigint(i)) => {
                if i.is_zero() {
                    (*ZERO_PCT).into()
                } else {
                    OwnedStringVar::from_str_checked(format!("{}00%", i))
                }
            }
            Variable::Normal(InnerVar::Bool(b)) => if b { *HUNDRED_PCT } else { *ZERO_PCT }.into(),
            Variable::Normal(InnerVar::Decimal(c)) => {
                let normalized = BigRational::from(BigInt::from(100)) * &*c;
                let rounded = normalized.round();
                if rounded.is_zero() {
                    (*ZERO_PCT).into()
                } else {
                    debug_assert!(rounded.denom().is_one());
                    OwnedStringVar::from_str_checked(format!("{}%", rounded.numer()))
                }
            }
            _ => panic!(),
        }
    }
}

impl Align {
    pub fn from_u8(x: u8) -> Align {
        match x as char {
            '<' => Align::Left,
            '>' => Align::Right,
            '^' => Align::Center,
            '=' => Align::AfterSign,
            x => panic!("Invalid align type: {} (hex value {:x})", x, x as u32),
        }
    }
}

impl Sign {
    pub fn from_u8(x: u8) -> Sign {
        match x as char {
            '-' => Sign::NegativeOnly,
            '+' => Sign::Both,
            ' ' => Sign::LeadingSpace,
            x => panic!("Invalid sign type: {} (hex value {:x})", x, x as u32),
        }
    }
}

impl FmtType {
    pub fn from_u8(x: u8) -> FmtType {
        match x as char {
            'b' => FmtType::Binary,
            'c' => FmtType::Character,
            'd' => FmtType::Decimal,
            'o' => FmtType::Octal,
            'x' => FmtType::Hex,
            'X' => FmtType::UpperHex,
            'n' => FmtType::Number,
            'e' => FmtType::Exponent,
            'E' => FmtType::UpperExp,
            'f' => FmtType::Fixed,
            'F' => FmtType::UpperFixed,
            'g' => FmtType::General,
            'G' => FmtType::UpperGeneral,
            '%' => FmtType::Percentage,
            'r' => FmtType::Repr,
            's' => FmtType::String,
            x => panic!("Invalid format type: {}", x),
        }
    }
}

impl Display for Align {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Align::Left => '<',
            Align::Right => '>',
            Align::AfterSign => '=',
            Align::Center => '^',
        })
    }
}

impl Default for Align {
    fn default() -> Self {
        Align::Left
    }
}

impl Display for Sign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Sign::Both => '+',
            Sign::NegativeOnly => '-',
            Sign::LeadingSpace => ' ',
        })
    }
}

impl Default for Sign {
    fn default() -> Self {
        Sign::NegativeOnly
    }
}

impl Display for FmtType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            FmtType::Binary => 'b',
            FmtType::Character => 'c',
            FmtType::Decimal => 'd',
            FmtType::Octal => 'o',
            FmtType::Hex => 'x',
            FmtType::UpperHex => 'X',
            FmtType::Number => 'n',
            FmtType::Exponent => 'e',
            FmtType::UpperExp => 'E',
            FmtType::Fixed => 'f',
            FmtType::UpperFixed => 'F',
            FmtType::General => 'g',
            FmtType::UpperGeneral => 'G',
            FmtType::Percentage => '%',
            FmtType::Repr => 'r',
            FmtType::String => 's',
        })
    }
}

impl Default for FmtType {
    fn default() -> Self {
        FmtType::String
    }
}

// TODO? Use AsciiString instead of String
fn as_decimal(value: &BigRational, precision: usize) -> String {
    let ten_pow_prec = Pow::pow(&*BIG_TEN, precision);
    let rounded = (value.clone() * &ten_pow_prec).round();
    let minus = rounded.is_negative();
    let trunc = &((&rounded / &ten_pow_prec).trunc().to_integer());
    let tail = &(rounded % ten_pow_prec).abs().to_integer();
    let mut ret_val = String::new();
    if minus {
        ret_val.push('-');
    }
    ret_val.push_str(&trunc.to_string());
    ret_val.push('.');
    let tail_str = tail.to_string();
    let tail_length = tail_str.chars().count();
    let tail_zeroes = if tail_length < precision {
        "0".repeat(precision - tail_length)
    } else {
        String::new()
    };
    ret_val.push_str(&tail_zeroes);
    ret_val.push_str(&tail_str);
    ret_val
}

// Based on https://stackoverflow.com/questions/18828377/
fn digit_count(value: BigInt) -> u64 {
    if let Option::Some(small) = value.to_u64() {
        return small_digit_count(small);
    }
    let mut value = value.into_parts().1;
    let mut digits = 0;
    let mut bits = value.bits();
    while bits > 4 {
        // 4 > log[2](10) so we should not reduce it too far.
        let reduce = bits / 4;
        value /= Pow::pow(&*BIG_U_TEN, reduce);
        digits += reduce;
        bits = value.bits();
    }
    if bits.to_u64().unwrap() > 9 {
        digits += 1;
    }
    digits
}

fn small_digit_count(value: u64) -> u64 {
    todo!()
}

fn get_formatter(var: Variable) -> Rc<FormatArgs> {
    match var {
        Variable::Normal(InnerVar::Custom(c)) => c.into_inner().downcast_rc().unwrap(),
        _ => panic!(),
    }
}

impl CustomVar for FormatArgs {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        todo!()
    }

    fn get_operator(self: Rc<Self>, _op: Operator) -> Variable {
        unimplemented!()
    }

    fn get_attribute(self: Rc<Self>, _name: &str) -> Variable {
        unimplemented!()
    }
}
