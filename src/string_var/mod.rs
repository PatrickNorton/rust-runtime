mod chunks;
mod impls;
mod maybe;
mod owned;
mod vars;

pub use maybe::{MaybeAscii, MaybeString};
pub use owned::OwnedStringVar;
pub use vars::{AsciiVar, StrVar};

use crate::character;
use crate::string_var::chunks::{AsciiChunks, StrChunks};
use crate::string_var::impls::{MixedIter, OwnedIter};
use ascii::{AsAsciiStr, AsAsciiStrError, AsciiChar, AsciiStr, AsciiString};
use std::borrow::{Borrow, Cow};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum StringVar {
    Literal(&'static str),
    AsciiLiteral(&'static AsciiStr),
    Other(Arc<str>),
    Ascii(Arc<AsciiStr>),
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

    pub fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        match self.as_maybe_ascii() {
            MaybeAscii::Standard(s) => s.as_ascii_str(),
            MaybeAscii::Ascii(a) => Result::Ok(a),
        }
    }

    pub fn from_leak(var: String) -> StringVar {
        StringVar::Literal(Box::leak(var.into_boxed_str()))
    }

    pub fn from_leak_ascii(var: Box<[AsciiChar]>) -> StringVar {
        StringVar::AsciiLiteral(Box::leak(var.into()))
    }

    pub fn from_str_ref(var: &str) -> StringVar {
        StringVar::Other(var.into())
    }

    pub fn from_ascii_ref(var: &AsciiStr) -> StringVar {
        let arc = Arc::<[AsciiChar]>::from(var.as_slice());
        // SAFETY: The internal representation of AsciiStr is the same as [AsciiChar]
        // This is the same as Arc::from()
        let arc = unsafe { Arc::from_raw(Arc::into_raw(arc) as *const AsciiStr) };
        StringVar::Ascii(arc)
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

    pub fn as_owned(&self) -> MaybeString {
        match self.as_maybe_ascii() {
            MaybeAscii::Standard(s) => MaybeString::Standard(s.to_owned()),
            MaybeAscii::Ascii(a) => MaybeString::Ascii(a.to_owned()),
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = MaybeAscii<'_>> {
        match self.as_maybe_ascii() {
            MaybeAscii::Standard(s) => MixedIter::Normal(s.lines()),
            MaybeAscii::Ascii(a) => MixedIter::Ascii(a.lines()),
        }
    }

    pub fn owned_lines(&self) -> impl Iterator<Item = StringVar> + '_ {
        match self {
            StringVar::Literal(s) => OwnedIter::Literal(s.lines()),
            StringVar::AsciiLiteral(a) => OwnedIter::AsciiLiteral(a.lines()),
            StringVar::Other(s) => OwnedIter::Normal(s.lines()),
            StringVar::Ascii(a) => OwnedIter::Ascii(a.lines()),
        }
    }

    pub fn chunks(&self, count: usize) -> impl Iterator<Item = MaybeAscii<'_>> {
        match self.as_maybe_ascii() {
            MaybeAscii::Standard(s) => MixedIter::Normal(StrChunks::new(s, count)),
            MaybeAscii::Ascii(a) => MixedIter::Ascii(AsciiChunks::new(a, count)),
        }
    }

    pub fn repr(&self) -> StringVar {
        let x: String = self.chars().map(character::repr).collect();
        StringVar::from(format!("\"{}\"", x))
    }

    pub fn is_ascii(&self) -> bool {
        match self.as_maybe_ascii() {
            MaybeAscii::Standard(s) => s.is_ascii(),
            MaybeAscii::Ascii(_) => true,
        }
    }

    pub fn to_uppercase(&self) -> String {
        match self.as_maybe_ascii() {
            MaybeAscii::Standard(s) => s.to_uppercase(),
            MaybeAscii::Ascii(a) => a.as_str().to_ascii_uppercase(),
        }
    }

    pub fn to_lowercase(&self) -> String {
        match self.as_maybe_ascii() {
            MaybeAscii::Standard(s) => s.to_lowercase(),
            MaybeAscii::Ascii(a) => a.as_str().to_ascii_lowercase(),
        }
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

impl From<&'static AsciiStr> for StringVar {
    fn from(x: &'static AsciiStr) -> Self {
        StringVar::AsciiLiteral(x)
    }
}

impl From<String> for StringVar {
    fn from(x: String) -> Self {
        StringVar::Other(Arc::from(x))
    }
}

impl From<AsciiString> for StringVar {
    fn from(x: AsciiString) -> Self {
        let values: Vec<u8> = x.into();
        let arc = Arc::<[u8]>::from(values.into_boxed_slice());
        // SAFETY: The internal representation of AsciiStr is the same as [u8]
        // This is the same as Arc::from()
        let arc = unsafe { Arc::from_raw(Arc::into_raw(arc) as *const AsciiStr) };
        StringVar::Ascii(arc)
    }
}

impl From<MaybeString> for StringVar {
    fn from(s: MaybeString) -> Self {
        match s {
            MaybeString::Standard(s) => StringVar::Other(s.into()),
            MaybeString::Ascii(a) => a.into(),
        }
    }
}

impl From<StrVar> for StringVar {
    fn from(s: StrVar) -> Self {
        match s {
            StrVar::Literal(l) => StringVar::Literal(l),
            StrVar::Other(o) => StringVar::Other(o),
        }
    }
}

impl From<AsciiVar> for StringVar {
    fn from(s: AsciiVar) -> Self {
        match s {
            AsciiVar::Literal(l) => StringVar::AsciiLiteral(l),
            AsciiVar::Other(o) => StringVar::Ascii(o),
        }
    }
}

impl From<OwnedStringVar> for StringVar {
    fn from(s: OwnedStringVar) -> Self {
        match s {
            OwnedStringVar::Literal(l) => StringVar::Literal(l),
            OwnedStringVar::AsciiLiteral(a) => StringVar::AsciiLiteral(a),
            OwnedStringVar::Other(s) => s.into(),
            OwnedStringVar::Ascii(a) => a.into(),
        }
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
