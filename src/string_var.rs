use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum StringVar {
    Literal(&'static str),
    Other(Arc<str>),
}

impl StringVar {
    pub fn as_str(&self) -> &str {
        match self {
            StringVar::Literal(a) => *a,
            StringVar::Other(x) => &x,
        }
    }

    pub fn from_leak(var: String) -> StringVar {
        StringVar::Literal(Box::leak(var.into_boxed_str()))
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
            StringVar::Other(s) => s.as_ref(),
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
