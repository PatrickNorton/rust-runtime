use crate::custom_types::exceptions::value_error;
use crate::custom_var::CustomVar;
use crate::first_n;
use crate::fmt_num::{
    format_int_exp, format_int_upper_exp, format_rational_unsigned, format_u_exp,
    format_upper_u_exp,
};
use crate::from_bool::FromBool;
use crate::int_tools::bytes_index;
use crate::int_var::IntVar;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::{OwnedStringVar, StringVar};
use crate::variable::{FnResult, InnerVar, Variable};
use ascii::{AsAsciiStr, AsciiChar, AsciiStr};
use num::{bigint, BigInt, BigRational, BigUint, One, ToPrimitive, Zero};
use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::fmt::{Display, Formatter, Write};
use std::rc::Rc;

const DEFAULT_FLOAT_DECIMALS: usize = 6;

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
            FmtType::Character => self.fmt_character(arg, runtime).map(From::from),
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
            FmtType::Repr => self.fmt_repr(arg, runtime),
            FmtType::String => self.fmt_str(arg, runtime),
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

    fn is_default_float(&self) -> bool {
        self.precision == 0 || self.precision == DEFAULT_FLOAT_DECIMALS as u32
    }

    fn float_decimals(&self) -> u32 {
        if self.precision == 0 {
            DEFAULT_FLOAT_DECIMALS as u32
        } else {
            self.precision
        }
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
        self.pad_str_simple(value, sign_chr, prefix)
    }

    fn pad_str_simple(
        &self,
        mut value: OwnedStringVar,
        sign_chr: Option<char>,
        prefix: &str,
    ) -> OwnedStringVar {
        if let Option::Some(sign) = sign_chr {
            value.insert(0, sign);
        }
        if let Option::Some(diff) = (self.min_width as usize).checked_sub(value.char_len()) {
            if self.zero && self.fill == '\0' {
                let start = if sign_chr.is_some() { 1 } else { 0 }
                    + if self.hash { prefix.len() } else { 0 };
                value.insert_n_chr(start, diff, '0');
            } else {
                let fill_char = self.fill_char();
                match self.align {
                    Align::Left => value.insert_n_chr(0, diff, fill_char),
                    Align::Right => value.push_n_chr(diff, fill_char),
                    Align::AfterSign => {
                        value.insert_n_chr(usize::from_bool(sign_chr.is_some()), diff, fill_char)
                    }
                    Align::Center => {
                        let pre_count = diff / 2; // Rounds down
                        let post_count = (diff + 1) / 2; // Rounds up
                        value.insert_n_chr(0, pre_count, fill_char);
                        value.push_n_chr(post_count, fill_char);
                    }
                }
            }
            value
        } else {
            value
        }
    }

    fn fill_char(&self) -> char {
        if self.fill == '\0' {
            ' '
        } else {
            self.fill
        }
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

    fn fmt_str(&self, var: Variable, runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.fmt_string_like(var.str(runtime)?))
    }

    fn fmt_repr(&self, var: Variable, runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.fmt_string_like(var.repr(runtime)?))
    }

    fn fmt_string_like(&self, value: StringVar) -> StringVar {
        if self.is_simple_format() {
            value
        } else {
            let owned = OwnedStringVar::from(value);
            self.pad_str_simple(owned, Option::None, "").into()
        }
    }

    fn fmt_binary(&self, var: Variable) -> OwnedStringVar {
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{:b}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "0b")
    }

    fn fmt_decimal(&self, var: Variable) -> OwnedStringVar {
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "")
    }

    fn fmt_octal(&self, var: Variable) -> OwnedStringVar {
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{:o}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "0o")
    }

    fn fmt_hex(&self, var: Variable) -> OwnedStringVar {
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{:x}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "0x")
    }

    fn fmt_upper_hex(&self, var: Variable) -> OwnedStringVar {
        let value = IntVar::from(var);
        let str_val = OwnedStringVar::from_str_checked(format!("{:X}", value.magnitude()));
        self.pad_integer(str_val, value.sign(), "0X")
    }

    fn fmt_character(&self, var: Variable, runtime: &mut Runtime) -> Result<OwnedStringVar, ()> {
        assert!(!self.zero);
        let char_str = match var {
            Variable::Normal(InnerVar::Bigint(i)) => match Self::char_from_int(&i) {
                Option::Some(x) => x,
                Option::None => return runtime.throw_quick_native(
                    value_error(),
                    format!(
                        "Invalid argument to !c string format: Scalar value {} is not a Unicode character value", 
                        i
                    )
                )
            }
            Variable::Normal(InnerVar::Bool(b)) => Self::char_from_bool(b),
            Variable::Normal(InnerVar::Char(c)) => c.into(),
            x => panic!(
                "Attempted to turn a variable not a superclass of int ({}) into an int",
                x.get_type().str()
            ),
        };
        Result::Ok(self.pad_str_simple(char_str, Option::None, ""))
    }

    fn char_from_int(i: &IntVar) -> Option<OwnedStringVar> {
        i.to_u32().and_then(char::from_u32).map(From::from)
    }

    fn char_from_bool(i: bool) -> OwnedStringVar {
        if i { AsciiChar::SOH } else { AsciiChar::Null }.into()
    }

    fn fmt_number(&self, var: Variable) -> OwnedStringVar {
        self.fmt_decimal(var)
    }

    fn fmt_exp(&self, var: Variable) -> OwnedStringVar {
        match var {
            Variable::Normal(InnerVar::Bigint(i)) => self.fmt_integer_exp(&i, false),
            Variable::Normal(InnerVar::Bool(b)) => self.fmt_bool_exp(b, false),
            Variable::Normal(InnerVar::Decimal(c)) => self.fmt_exp_inner(&*c, false),
            _ => panic!(),
        }
    }

    fn fmt_upper_exp(&self, var: Variable) -> OwnedStringVar {
        match var {
            Variable::Normal(InnerVar::Bigint(i)) => self.fmt_integer_exp(&i, true),
            Variable::Normal(InnerVar::Bool(b)) => self.fmt_bool_exp(b, true),
            Variable::Normal(InnerVar::Decimal(c)) => self.fmt_exp_inner(&*c, true),
            _ => panic!(),
        }
    }

    fn fmt_integer_exp(&self, value: &IntVar, uppercase: bool) -> OwnedStringVar {
        if value.is_zero() {
            self.fmt_zero_exp(uppercase)
        } else {
            let formatted = if uppercase {
                format_int_upper_exp(&*int_var_as_magnitude(value), self.float_decimals())
            } else {
                format_int_exp(&*int_var_as_magnitude(value), self.float_decimals())
            };
            let sign = self.sign_char(value.sign());
            self.pad_str_simple(OwnedStringVar::from_str_checked(formatted), sign, "")
        }
    }

    fn fmt_bool_exp(&self, value: bool, uppercase: bool) -> OwnedStringVar {
        if !value {
            self.fmt_zero_exp(uppercase)
        } else {
            self.fmt_one_exp(uppercase)
        }
    }

    fn fmt_exp_inner(&self, value: &BigRational, uppercase: bool) -> OwnedStringVar {
        if value.is_zero() {
            self.fmt_zero_exp(uppercase)
        } else {
            let str = OwnedStringVar::from_str_checked(if uppercase {
                format_upper_u_exp(value.clone(), self.float_decimals())
            } else {
                format_u_exp(value.clone(), self.float_decimals())
            });
            let sign = self.sign_char(value.numer().sign());
            self.pad_str_simple(str, sign, "")
        }
    }

    fn fmt_zero_exp(&self, uppercase: bool) -> OwnedStringVar {
        static SIMPLE_LOWER: Lazy<&AsciiStr> = Lazy::new(|| "0.000000e+00".as_ascii_str().unwrap());
        static SIMPLE_UPPER: Lazy<&AsciiStr> = Lazy::new(|| "0.000000E+00".as_ascii_str().unwrap());
        if self.is_simple_format() {
            if uppercase {
                (*SIMPLE_UPPER).into()
            } else {
                (*SIMPLE_LOWER).into()
            }
        } else {
            let e = if uppercase { 'E' } else { 'e' };
            let precision = self.float_decimals() as usize;
            let sign = self.sign_char(bigint::Sign::NoSign);
            let str = format!("0.{:0width$}{}+00", 0, e, width = precision);
            self.pad_str_simple(OwnedStringVar::from_str_checked(str), sign, "")
        }
    }

    fn fmt_one_exp(&self, uppercase: bool) -> OwnedStringVar {
        static SIMPLE_LOWER: Lazy<&AsciiStr> = Lazy::new(|| "1.000000e+00".as_ascii_str().unwrap());
        static SIMPLE_UPPER: Lazy<&AsciiStr> = Lazy::new(|| "1.000000E+00".as_ascii_str().unwrap());
        if self.is_simple_format() {
            if uppercase {
                (*SIMPLE_UPPER).into()
            } else {
                (*SIMPLE_LOWER).into()
            }
        } else {
            let e = if uppercase { 'E' } else { 'e' };
            let precision = self.float_decimals() as usize;
            let sign = self.sign_char(bigint::Sign::NoSign);
            let str = format!("1.{:0width$}{}+00", 0, e, width = precision);
            self.pad_str_simple(OwnedStringVar::from_str_checked(str), sign, "")
        }
    }

    fn fmt_fixed(&self, var: Variable) -> OwnedStringVar {
        self.fixed_inner(var, false)
    }

    fn fmt_upper_fixed(&self, var: Variable) -> OwnedStringVar {
        self.fixed_inner(var, true)
    }

    fn fixed_inner(&self, var: Variable, _uppercase: bool) -> OwnedStringVar {
        static ZERO_FIXED: Lazy<&AsciiStr> =
            Lazy::new(|| AsciiStr::from_ascii("0.000000").unwrap());
        static ONE_FIXED: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii("1.000000").unwrap());
        match var {
            Variable::Normal(InnerVar::Bigint(i)) => {
                if i.is_zero() && self.is_default_float() {
                    (*ZERO_FIXED).into()
                } else {
                    let precision = self.float_decimals() as usize;
                    let sign_char = self.sign_char(i.sign());
                    let value =
                        format!("{}.{:precision$}", i.magnitude(), 0, precision = precision);
                    self.pad_str_simple(OwnedStringVar::from_str_checked(value), sign_char, "")
                }
            }
            Variable::Normal(InnerVar::Bool(b)) => {
                if self.is_default_float() {
                    if b { *ONE_FIXED } else { *ZERO_FIXED }.into()
                } else {
                    let width = self.float_decimals() as usize;
                    let value = format!("{:.*}", width, f32::from_bool(b));
                    self.pad_str_simple(
                        OwnedStringVar::from_str_checked(value),
                        self.sign_char(bool_sign(b)),
                        "",
                    )
                }
            }
            Variable::Normal(InnerVar::Decimal(d)) => {
                let is_default = self.is_default_float();
                if is_default && d.is_zero() {
                    (*ZERO_FIXED).into()
                } else if is_default && d.is_one() {
                    (*ONE_FIXED).into()
                } else {
                    let value = OwnedStringVar::from_str_checked(format_rational_unsigned(
                        (*d).clone(),
                        self.float_decimals(),
                    ));
                    self.pad_str_simple(value, self.sign_char(d.sign()), "")
                }
            }
            _ => panic!(),
        }
    }

    fn fmt_general(&self, _var: Variable) -> OwnedStringVar {
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
            '>' => Align::Left,
            '<' => Align::Right,
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

fn get_formatter(var: Variable) -> Rc<FormatArgs> {
    match var {
        Variable::Normal(InnerVar::Custom(c)) => c.into_inner().downcast_rc().unwrap(),
        _ => panic!(),
    }
}

fn bool_sign(x: bool) -> bigint::Sign {
    if x {
        bigint::Sign::Plus
    } else {
        bigint::Sign::NoSign
    }
}

fn int_var_as_magnitude(x: &IntVar) -> Cow<'_, BigUint> {
    match x {
        IntVar::Small(s) => Cow::Owned(s.unsigned_abs().into()),
        IntVar::Big(b) => Cow::Borrowed(b.magnitude()),
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

#[cfg(test)]
mod test {
    use crate::fmt::{Align, FmtType, FormatArgs, Sign};
    use crate::rational_var::RationalVar;
    use num::{BigInt, BigRational, One};

    #[test]
    fn simple_binary() {
        let formatter = FormatArgs::default();
        assert_eq!(&*formatter.fmt_binary(0b101.into()), "101");
    }

    #[test]
    fn prefix() {
        let formatter = FormatArgs {
            hash: true,
            fmt_type: FmtType::Binary,
            ..Default::default()
        };
        assert_eq!(&*formatter.fmt_binary(0b101.into()), "0b101");
    }

    #[test]
    fn simple_octal() {
        let formatter = FormatArgs::default();
        assert_eq!(&*formatter.fmt_octal(0o101.into()), "101");
    }

    #[test]
    fn padding() {
        let formatter = FormatArgs {
            fill: ':',
            min_width: 5,
            fmt_type: FmtType::Octal,
            ..Default::default()
        };
        assert_eq!(&*formatter.fmt_octal(0o101.into()), "::101")
    }

    #[test]
    fn pad_post() {
        let formatter = FormatArgs {
            fill: ':',
            min_width: 5,
            align: Align::Right,
            fmt_type: FmtType::Octal,
            ..Default::default()
        };
        assert_eq!(&*formatter.fmt_octal(0o101.into()), "101::")
    }

    #[test]
    fn pad_mid() {
        let formatter = FormatArgs {
            fill: ':',
            align: Align::Center,
            min_width: 5,
            fmt_type: FmtType::Octal,
            ..Default::default()
        };
        assert_eq!(&*formatter.fmt_octal(0o101.into()), ":101:")
    }

    #[test]
    fn simple_decimal() {
        let formatter = FormatArgs::default();
        assert_eq!(&*formatter.fmt_decimal(101.into()), "101");
    }

    #[test]
    fn pad_zeroes() {
        let formatter = FormatArgs {
            min_width: 7,
            zero: true,
            fmt_type: FmtType::Decimal,
            ..Default::default()
        };
        assert_eq!(&*formatter.fmt_decimal((-101).into()), "-000101")
    }

    #[test]
    fn pad_post_sign() {
        let formatter = FormatArgs {
            fill: ':',
            min_width: 7,
            align: Align::AfterSign,
            fmt_type: FmtType::Decimal,
            ..Default::default()
        };
        assert_eq!(&*formatter.fmt_decimal((-101).into()), "-:::101")
    }

    #[test]
    fn simple_hex() {
        let formatter = FormatArgs::default();
        assert_eq!(&*formatter.fmt_hex(0x10a.into()), "10a");
    }

    #[test]
    fn positive_sign() {
        let formatter = FormatArgs {
            sign: Sign::Both,
            fmt_type: FmtType::Hex,
            ..Default::default()
        };
        assert_eq!(&*formatter.fmt_hex(0x10a.into()), "+10a");
    }

    #[test]
    fn simple_upper_hex() {
        let formatter = FormatArgs::default();
        assert_eq!(&*formatter.fmt_upper_hex(0x10A.into()), "10A");
    }

    #[test]
    fn space_sign() {
        let formatter = FormatArgs {
            sign: Sign::LeadingSpace,
            fmt_type: FmtType::Hex,
            ..Default::default()
        };
        assert_eq!(&*formatter.fmt_hex(0x10a.into()), " 10a");
    }

    #[test]
    fn simple_exp() {
        let formatter = FormatArgs::default();
        let third = BigRational::new(BigInt::one(), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_exp(RationalVar::from(third).into()),
            "3.333333e-01"
        )
    }

    #[test]
    fn simple_upper_exp() {
        let formatter = FormatArgs::default();
        let third = BigRational::new(BigInt::one(), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_upper_exp(RationalVar::from(third).into()),
            "3.333333E-01"
        );
    }

    #[test]
    fn exp_sign() {
        let formatter = FormatArgs {
            sign: Sign::Both,
            fmt_type: FmtType::Exponent,
            ..Default::default()
        };
        let third = BigRational::new(BigInt::from(1), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_exp(RationalVar::from(third).into()),
            "+3.333333e-01"
        );
        let neg_third = BigRational::new(BigInt::from(-1), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_exp(RationalVar::from(neg_third).into()),
            "-3.333333e-01"
        );
    }

    #[test]
    fn simple_fixed() {
        let formatter = FormatArgs::default();
        let third = BigRational::new(BigInt::one(), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_fixed(RationalVar::from(third).into()),
            "0.333333"
        );
    }

    #[test]
    fn fixed_sign() {
        let formatter = FormatArgs {
            sign: Sign::Both,
            fmt_type: FmtType::Fixed,
            ..Default::default()
        };
        let third = BigRational::new(BigInt::from(10), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_fixed(RationalVar::from(third).into()),
            "+3.333333"
        );
        let neg_third = BigRational::new(BigInt::from(-10), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_fixed(RationalVar::from(neg_third).into()),
            "-3.333333"
        );
    }

    #[test]
    fn zero_fixed() {
        let formatter = FormatArgs {
            zero: true,
            min_width: 10,
            fmt_type: FmtType::Fixed,
            ..Default::default()
        };
        let third = BigRational::new(BigInt::from(10), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_fixed(RationalVar::from(third).into()),
            "003.333333"
        );
        let neg_third = BigRational::new(BigInt::from(-10), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_fixed(RationalVar::from(neg_third).into()),
            "-03.333333"
        );
    }

    #[test]
    fn simple_pct() {
        let formatter = FormatArgs::default();
        let third = BigRational::new(BigInt::one(), BigInt::from(3));
        assert_eq!(
            &*formatter.fmt_percentage(RationalVar::from(third).into()),
            "33%"
        );
    }
}
