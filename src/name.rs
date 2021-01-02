use crate::operator::Operator;
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Name<'a> {
    Attribute(&'a str),
    Operator(Operator),
}

impl<'a> Name<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            Name::Attribute(s) => *s,
            Name::Operator(o) => o.name(),
        }
    }

    pub fn do_each<T>(self, op: impl FnOnce(Operator) -> T, attr: impl FnOnce(&str) -> T) -> T {
        match self {
            Name::Attribute(s) => attr(s),
            Name::Operator(o) => op(o),
        }
    }
}

impl<'a> std::fmt::Display for Name<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Name::Attribute(s) => s.fmt(f),
            Name::Operator(o) => o.name().fmt(f),
        }
    }
}

impl<'a> From<Operator> for Name<'a> {
    fn from(x: Operator) -> Self {
        Name::Operator(x)
    }
}

impl<'a> From<&'a str> for Name<'a> {
    fn from(x: &'a str) -> Self {
        Name::Attribute(x)
    }
}
