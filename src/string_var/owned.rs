use crate::string_var::{AsciiVar, MaybeString, StrVar};
use ascii::{AsciiStr, AsciiString};
use std::borrow::Cow;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub enum OwnedStringVar {
    Literal(&'static str),
    AsciiLiteral(&'static AsciiStr),
    Other(String),
    Ascii(AsciiString),
}

impl OwnedStringVar {
    pub fn from_str_checked(value: String) -> OwnedStringVar {
        match AsciiString::from_ascii(value) {
            Result::Ok(x) => OwnedStringVar::Ascii(x),
            Result::Err(e) => OwnedStringVar::Other(e.into_source()),
        }
    }

    pub fn make_ascii_uppercase(&mut self) {
        match self {
            OwnedStringVar::Literal(l) => {
                *self = OwnedStringVar::Other(l.to_ascii_uppercase());
            }
            OwnedStringVar::AsciiLiteral(a) => {
                *self = OwnedStringVar::Ascii(a.to_ascii_uppercase());
            }
            OwnedStringVar::Other(o) => o.make_ascii_uppercase(),
            OwnedStringVar::Ascii(a) => a.make_ascii_uppercase(),
        }
    }
}

impl From<&'static str> for OwnedStringVar {
    fn from(x: &'static str) -> Self {
        OwnedStringVar::Literal(x)
    }
}

impl From<Cow<'static, str>> for OwnedStringVar {
    fn from(x: Cow<'static, str>) -> Self {
        match x {
            Cow::Borrowed(x) => OwnedStringVar::Literal(x),
            Cow::Owned(x) => OwnedStringVar::Other(x.into()),
        }
    }
}

impl From<&'static AsciiStr> for OwnedStringVar {
    fn from(x: &'static AsciiStr) -> Self {
        OwnedStringVar::AsciiLiteral(x)
    }
}

impl From<String> for OwnedStringVar {
    fn from(x: String) -> Self {
        OwnedStringVar::Other(x)
    }
}

impl From<AsciiString> for OwnedStringVar {
    fn from(x: AsciiString) -> Self {
        OwnedStringVar::Ascii(x)
    }
}

impl From<MaybeString> for OwnedStringVar {
    fn from(s: MaybeString) -> Self {
        match s {
            MaybeString::Standard(s) => OwnedStringVar::Other(s.into()),
            MaybeString::Ascii(a) => a.into(),
        }
    }
}

impl From<StrVar> for OwnedStringVar {
    fn from(s: StrVar) -> Self {
        match s {
            StrVar::Literal(l) => OwnedStringVar::Literal(l),
            StrVar::Other(o) => OwnedStringVar::Other(o.to_string()),
        }
    }
}

impl From<AsciiVar> for OwnedStringVar {
    fn from(s: AsciiVar) -> Self {
        match s {
            AsciiVar::Literal(l) => OwnedStringVar::AsciiLiteral(l),
            AsciiVar::Other(o) => OwnedStringVar::Ascii(o.to_ascii_string()),
        }
    }
}

impl Deref for OwnedStringVar {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            OwnedStringVar::Literal(l) => *l,
            OwnedStringVar::AsciiLiteral(a) => a.as_str(),
            OwnedStringVar::Other(o) => &o,
            OwnedStringVar::Ascii(a) => a.as_str(),
        }
    }
}
