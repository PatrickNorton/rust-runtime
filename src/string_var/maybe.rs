use crate::string_var::StringVar;
use ascii::{AsciiStr, AsciiString};
use std::mem::take;
use std::ops::AddAssign;

#[derive(Debug, Copy, Clone)]
pub enum MaybeAscii<'a> {
    Standard(&'a str),
    Ascii(&'a AsciiStr),
}

#[derive(Debug)]
pub enum MaybeString {
    Standard(String),
    Ascii(AsciiString),
}

impl<'a> MaybeAscii<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            MaybeAscii::Standard(s) => *s,
            MaybeAscii::Ascii(a) => a.as_str(),
        }
    }
}

impl MaybeString {
    pub fn new() -> MaybeString {
        Default::default()
    }
}

impl AddAssign<MaybeAscii<'_>> for MaybeString {
    fn add_assign(&mut self, rhs: MaybeAscii<'_>) {
        match self {
            MaybeString::Standard(s) => *s += rhs.as_str(),
            MaybeString::Ascii(a) => match rhs {
                MaybeAscii::Standard(s) => {
                    let mut val: String = take(a).into();
                    val += s;
                    *self = MaybeString::Standard(val)
                }
                MaybeAscii::Ascii(s) => *a += s,
            },
        }
    }
}

impl AddAssign<&'_ AsciiStr> for MaybeString {
    fn add_assign(&mut self, rhs: &AsciiStr) {
        match self {
            MaybeString::Standard(s) => *s += rhs.as_str(),
            MaybeString::Ascii(a) => *a += rhs,
        }
    }
}

impl AddAssign<&'_ StringVar> for MaybeString {
    fn add_assign(&mut self, rhs: &StringVar) {
        *self += rhs.as_maybe_ascii()
    }
}

impl Default for MaybeString {
    fn default() -> Self {
        MaybeString::Ascii(AsciiString::default())
    }
}
