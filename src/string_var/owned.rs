use crate::string_var::{AsciiVar, MaybeString, StrVar};
use ascii::{AsAsciiStr, AsciiStr, AsciiString, ToAsciiChar};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
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

    pub fn char_len(&self) -> usize {
        match self {
            OwnedStringVar::Literal(l) => l.chars().count(),
            OwnedStringVar::AsciiLiteral(a) => a.len(),
            OwnedStringVar::Other(o) => o.chars().count(),
            OwnedStringVar::Ascii(a) => a.len(),
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

    pub fn insert(&mut self, idx: usize, value: char) {
        match self {
            OwnedStringVar::Literal(l) => {
                let mut val = l.to_string();
                val.insert(idx, value);
                *self = OwnedStringVar::Other(val);
            }
            OwnedStringVar::AsciiLiteral(a) => {
                if let Result::Ok(value) = value.to_ascii_char() {
                    let mut val = a.to_ascii_string();
                    val.insert(idx, value);
                    *self = OwnedStringVar::Ascii(val);
                } else {
                    let mut val = a.to_string();
                    val.insert(idx, value);
                    *self = OwnedStringVar::Other(val);
                }
            }
            OwnedStringVar::Other(o) => o.insert(idx, value),
            OwnedStringVar::Ascii(a) => {
                if let Result::Ok(value) = value.to_ascii_char() {
                    a.insert(idx, value);
                } else {
                    let mut val = a.to_string();
                    val.insert(idx, value);
                    *self = OwnedStringVar::Other(val);
                }
            }
        }
    }

    pub fn insert_str(&mut self, idx: usize, value: &str) {
        match self {
            OwnedStringVar::Literal(l) => {
                let mut val = l.to_string();
                val.insert_str(idx, value);
                *self = OwnedStringVar::Other(val);
            }
            OwnedStringVar::AsciiLiteral(a) => {
                if let Result::Ok(value) = value.as_ascii_str() {
                    let mut val = a.to_ascii_string();
                    for (i, chr) in value.chars().enumerate() {
                        val.insert(idx + i, chr);
                    }
                    *self = OwnedStringVar::Ascii(val);
                } else {
                    let mut val = a.to_string();
                    val.insert_str(idx, value);
                    *self = OwnedStringVar::Other(val);
                }
            }
            OwnedStringVar::Other(o) => o.insert_str(idx, value),
            OwnedStringVar::Ascii(a) => {
                if let Result::Ok(value) = value.as_ascii_str() {
                    for (i, chr) in value.chars().enumerate() {
                        a.insert(idx + i, chr);
                    }
                } else {
                    let mut val = a.to_string();
                    val.insert_str(idx, value);
                    *self = OwnedStringVar::Other(val);
                }
            }
        }
    }

    pub fn insert_n_chr(&mut self, idx: usize, n: usize, value: char) {
        match self {
            OwnedStringVar::Literal(l) => {
                let mut val = l.to_string();
                val.insert_str(idx, &value.to_string().repeat(n));
                *self = OwnedStringVar::Other(val)
            }
            OwnedStringVar::AsciiLiteral(a) => {
                if let Result::Ok(value) = value.to_ascii_char() {
                    let mut val = a.to_ascii_string();
                    val.reserve(n);
                    for idx in idx..idx + n {
                        val.insert(idx, value);
                    }
                    *self = OwnedStringVar::Ascii(val)
                } else {
                    let mut val = a.to_string();
                    val.insert_str(idx, &value.to_string().repeat(n));
                    *self = OwnedStringVar::Other(val)
                }
            }
            OwnedStringVar::Other(o) => {
                o.insert_str(idx, &value.to_string().repeat(n));
            }
            OwnedStringVar::Ascii(a) => {
                if let Result::Ok(value) = value.to_ascii_char() {
                    a.reserve(n);
                    for idx in idx..idx + n {
                        a.insert(idx, value);
                    }
                } else {
                    let mut val = a.to_string();
                    val.insert_str(idx, &value.to_string().repeat(n));
                    *self = OwnedStringVar::Other(val)
                }
            }
        }
    }

    pub fn push_n_chr(&mut self, n: usize, value: char) {
        match self {
            OwnedStringVar::Literal(l) => {
                let mut val = l.to_string();
                val.push_str(&value.to_string().repeat(n));
                *self = OwnedStringVar::Other(val)
            }
            OwnedStringVar::AsciiLiteral(a) => {
                if let Result::Ok(value) = value.to_ascii_char() {
                    let mut val = a.to_ascii_string();
                    val.reserve(n);
                    for _ in 0..n {
                        val.push(value);
                    }
                    *self = OwnedStringVar::Ascii(val)
                } else {
                    let mut val = a.to_string();
                    val.push_str(&value.to_string().repeat(n));
                    *self = OwnedStringVar::Other(val)
                }
            }
            OwnedStringVar::Other(o) => {
                o.push_str(&value.to_string().repeat(n));
            }
            OwnedStringVar::Ascii(a) => {
                if let Result::Ok(value) = value.to_ascii_char() {
                    a.reserve(n);
                    for _ in 0..n {
                        a.push(value);
                    }
                } else {
                    let mut val = a.to_string();
                    val.push_str(&value.to_string().repeat(n));
                    *self = OwnedStringVar::Other(val)
                }
            }
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

impl PartialEq for OwnedStringVar {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for OwnedStringVar {}

impl Display for OwnedStringVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OwnedStringVar::Literal(l) => l.fmt(f),
            OwnedStringVar::AsciiLiteral(a) => a.fmt(f),
            OwnedStringVar::Other(o) => o.fmt(f),
            OwnedStringVar::Ascii(a) => a.fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::string_var::OwnedStringVar;

    #[test]
    fn from_str_checked() {
        let a = OwnedStringVar::from_str_checked("abc".to_owned());
        assert!(matches!(a, OwnedStringVar::Ascii(_)));
        assert_eq!(&*a, "abc");
        let b = OwnedStringVar::from_str_checked("ábc".to_owned());
        assert!(matches!(b, OwnedStringVar::Other(_)));
        assert_eq!(&*b, "ábc".to_owned());
    }

    #[test]
    fn char_len() {
        let a = OwnedStringVar::from_str_checked("abc".to_owned());
        assert_eq!(a.char_len(), 3);
        let b = OwnedStringVar::from_str_checked("ábc".to_owned());
        assert_eq!(b.char_len(), 3);
    }

    #[test]
    fn ascii_uppercase() {
        let mut a = OwnedStringVar::from_str_checked("abc".to_owned());
        a.make_ascii_uppercase();
        assert_eq!(&*a, "ABC");
    }

    #[test]
    fn insert() {
        let mut a = OwnedStringVar::from_str_checked("acd".to_owned());
        a.insert(1, 'b');
        assert_eq!(&*a, "abcd");
    }

    #[test]
    fn insert_str() {
        let mut a = OwnedStringVar::from_str_checked("aef".to_owned());
        a.insert_str(1, "bcd");
        assert_eq!(&*a, "abcdef");
    }

    #[test]
    fn insert_n_chr() {
        let mut a = OwnedStringVar::from_str_checked("abc".to_owned());
        a.insert_n_chr(1, 3, '0');
        assert_eq!(&*a, "a000bc");
    }

    #[test]
    fn push_n_chr() {
        let mut a = OwnedStringVar::from_str_checked("abc".to_owned());
        a.push_n_chr(3, '0');
        assert_eq!(&*a, "abc000");
    }
}
