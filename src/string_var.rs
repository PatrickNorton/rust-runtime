use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StringVar {
    Literal(&'static str),
    Other(Arc<Box<str>>),
}

impl StringVar {
    pub fn as_str(&self) -> &str {
        return match self {
            StringVar::Literal(a) => *a,
            StringVar::Other(x) => &x,
        };
    }
}

impl From<&'static str> for StringVar {
    fn from(x: &'static str) -> Self {
        StringVar::Literal(x)
    }
}

impl From<String> for StringVar {
    fn from(x: String) -> Self {
        StringVar::Other(Arc::new(x.into_boxed_str()))
    }
}

impl Deref for StringVar {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        return match self {
            StringVar::Literal(s) => *s,
            StringVar::Other(s) => &s.as_ref(),
        };
    }
}

impl std::fmt::Display for StringVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&**self, f)
    }
}
