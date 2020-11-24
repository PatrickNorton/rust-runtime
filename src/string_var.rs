use ascii::{AsciiChar, AsciiStr};
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

pub enum MaybeAscii<'a> {
    Standard(&'a str),
    Ascii(&'a AsciiStr),
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
}

impl From<&'static str> for StringVar {
    fn from(x: &'static str) -> Self {
        StringVar::Literal(x)
    }
}

impl From<String> for StringVar {
    fn from(x: String) -> Self {
        StringVar::Other(Arc::from(x.as_str()))
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
        "".into()
    }
}
