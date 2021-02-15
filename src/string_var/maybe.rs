use crate::string_var::StringVar;
use ascii::{AsciiChar, AsciiStr, AsciiString};
use std::fmt::{self, Display, Formatter};
use std::mem::take;
use std::ops::{Add, AddAssign};

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

impl MaybeAscii<'_> {
    pub fn as_str(&self) -> &str {
        match self {
            MaybeAscii::Standard(s) => *s,
            MaybeAscii::Ascii(a) => a.as_str(),
        }
    }

    pub fn char_len(&self) -> usize {
        match self {
            MaybeAscii::Standard(s) => s.chars().count(),
            MaybeAscii::Ascii(a) => a.len(),
        }
    }
}

impl Display for MaybeAscii<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MaybeAscii::Standard(s) => f.write_str(s),
            MaybeAscii::Ascii(a) => f.write_str(a.as_str()),
        }
    }
}

impl MaybeString {
    pub fn new() -> MaybeString {
        Default::default()
    }

    pub fn borrow(&self) -> MaybeAscii<'_> {
        match self {
            MaybeString::Standard(s) => MaybeAscii::Standard(&s),
            MaybeString::Ascii(a) => MaybeAscii::Ascii(&a),
        }
    }

    pub fn insert_ascii(&mut self, idx: usize, ch: AsciiChar) {
        match self {
            MaybeString::Standard(s) => s.insert(idx, ch.as_char()),
            MaybeString::Ascii(a) => a.insert(idx, ch),
        }
    }

    pub fn insert_ascii_str(&mut self, idx: usize, str: &AsciiStr) {
        match self {
            MaybeString::Standard(s) => s.insert_str(idx, str.as_str()),
            MaybeString::Ascii(a) => {
                // FIXME: Get insert_str added to AsciiString
                if idx == 0 {
                    let mut new = str.to_owned();
                    new.push_str(&a);
                    *a = new;
                } else if idx == a.len() {
                    a.push_str(str);
                } else {
                    assert!(idx < a.len());
                    let mut result = a[..idx].to_owned();
                    result.push_str(str);
                    result.push_str(&a[idx..]);
                    *a = result;
                }
            }
        }
    }

    pub fn push_ascii(&mut self, ch: AsciiChar) {
        match self {
            MaybeString::Standard(s) => s.push(ch.as_char()),
            MaybeString::Ascii(a) => a.push(ch),
        }
    }
}

impl Display for MaybeString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MaybeString::Standard(s) => f.write_str(s),
            MaybeString::Ascii(a) => f.write_str(a.as_str()),
        }
    }
}

impl Add<MaybeAscii<'_>> for MaybeString {
    type Output = Self;

    fn add(mut self, rhs: MaybeAscii<'_>) -> Self::Output {
        self += rhs;
        self
    }
}

impl Add<&'_ StringVar> for MaybeString {
    type Output = Self;

    fn add(self, rhs: &StringVar) -> Self::Output {
        self + rhs.as_maybe_ascii()
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

impl AddAssign<&MaybeString> for MaybeString {
    fn add_assign(&mut self, rhs: &MaybeString) {
        *self += rhs.borrow();
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
