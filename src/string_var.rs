use crate::character;
use ascii::{AsciiChar, AsciiStr, AsciiString};
use downcast_rs::__std::borrow::Borrow;
use downcast_rs::__std::iter::FromIterator;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum StringVar {
    Literal(&'static str),
    AsciiLiteral(&'static AsciiStr),
    Other(Arc<str>),
    Ascii(Arc<AsciiStr>),
}

#[derive(Debug, Copy, Clone)]
pub enum MaybeAscii<'a> {
    Standard(&'a str),
    Ascii(&'a AsciiStr),
}

#[derive(Debug, Clone)]
pub enum AsciiVar {
    Literal(&'static AsciiStr),
    Other(Arc<AsciiStr>),
}

#[derive(Debug, Clone)]
pub enum StrVar {
    Literal(&'static str),
    Other(Arc<str>),
}

impl StringVar {
    pub fn as_str(&self) -> &str {
        match self {
            StringVar::Literal(a) => *a,
            StringVar::AsciiLiteral(a) => a.as_str(),
            StringVar::Other(x) => &x,
            StringVar::Ascii(x) => x.as_str(),
        }
    }

    pub fn from_leak(var: String) -> StringVar {
        StringVar::Literal(Box::leak(var.into_boxed_str()))
    }

    pub fn from_leak_ascii(var: Box<[AsciiChar]>) -> StringVar {
        StringVar::AsciiLiteral(Box::leak(var.into()))
    }

    pub fn char_at(&self, i: usize) -> Option<char> {
        match self {
            StringVar::Literal(l) => l.chars().nth(i),
            StringVar::AsciiLiteral(a) => Option::Some(a[i].as_char()),
            StringVar::Other(o) => o.chars().nth(i),
            StringVar::Ascii(a) => Option::Some(a[i].as_char()),
        }
    }

    pub fn char_len(&self) -> usize {
        match self {
            StringVar::Literal(l) => l.chars().count(),
            StringVar::AsciiLiteral(a) => a.len(),
            StringVar::Other(o) => o.chars().count(),
            StringVar::Ascii(a) => a.len(),
        }
    }

    pub fn as_maybe_ascii(&self) -> MaybeAscii<'_> {
        match self {
            StringVar::Literal(l) => MaybeAscii::Standard(l),
            StringVar::AsciiLiteral(a) => MaybeAscii::Ascii(a),
            StringVar::Other(o) => MaybeAscii::Standard(o),
            StringVar::Ascii(a) => MaybeAscii::Ascii(a),
        }
    }

    pub fn split_ascii(self) -> Result<AsciiVar, StrVar> {
        match self {
            StringVar::Literal(l) => Result::Err(StrVar::Literal(l)),
            StringVar::AsciiLiteral(a) => Result::Ok(AsciiVar::Literal(a)),
            StringVar::Other(o) => Result::Err(StrVar::Other(o)),
            StringVar::Ascii(a) => Result::Ok(AsciiVar::Other(a)),
        }
    }

    pub fn repr(&self) -> StringVar {
        let x: String = self.chars().map(character::repr).collect();
        StringVar::from(format!("\"{}\"", x))
    }
}

impl From<&'static str> for StringVar {
    fn from(x: &'static str) -> Self {
        StringVar::Literal(x)
    }
}

impl From<Cow<'static, str>> for StringVar {
    fn from(x: Cow<'static, str>) -> Self {
        match x {
            Cow::Borrowed(x) => StringVar::Literal(x),
            Cow::Owned(x) => StringVar::Other(x.into()),
        }
    }
}

impl From<String> for StringVar {
    fn from(x: String) -> Self {
        StringVar::Other(Arc::from(x))
    }
}

impl From<AsciiString> for StringVar {
    fn from(x: AsciiString) -> Self {
        let arc = Arc::<[u8]>::from(x.as_bytes());
        // SAFETY: The internal representation of AsciiStr is the same as [u8]
        // This is the same as Arc::from(
        let arc = unsafe { Arc::from_raw(Arc::into_raw(arc) as *const AsciiStr) };
        StringVar::Ascii(arc)
    }
}

impl Deref for StringVar {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            StringVar::Literal(s) => *s,
            StringVar::AsciiLiteral(s) => s.as_str(),
            StringVar::Other(s) => s.as_ref(),
            StringVar::Ascii(s) => s.as_str(),
        }
    }
}

impl Borrow<str> for StringVar {
    fn borrow(&self) -> &str {
        &*self
    }
}

impl PartialEq for StringVar {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for StringVar {}

impl Hash for StringVar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl std::fmt::Display for StringVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&**self, f)
    }
}

impl Default for StringVar {
    fn default() -> Self {
        StringVar::AsciiLiteral(Default::default())
    }
}

impl Deref for StrVar {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            StrVar::Literal(s) => *s,
            StrVar::Other(s) => s.as_ref(),
        }
    }
}

impl Deref for AsciiVar {
    type Target = AsciiStr;

    fn deref(&self) -> &Self::Target {
        match self {
            AsciiVar::Literal(s) => *s,
            AsciiVar::Other(s) => s.as_ref(),
        }
    }
}

impl FromIterator<String> for StringVar {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        iter.into_iter().collect::<String>().into()
    }
}

impl<'a> FromIterator<&'a str> for StringVar {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        iter.into_iter().collect::<String>().into()
    }
}

impl FromIterator<AsciiString> for StringVar {
    fn from_iter<T: IntoIterator<Item = AsciiString>>(iter: T) -> Self {
        iter.into_iter().collect::<AsciiString>().into()
    }
}

impl<'a> FromIterator<&'a AsciiStr> for StringVar {
    fn from_iter<T: IntoIterator<Item = &'a AsciiStr>>(iter: T) -> Self {
        iter.into_iter().collect::<AsciiString>().into()
    }
}
