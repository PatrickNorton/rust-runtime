use crate::operator::Operator;
use crate::string_var::StringVar;
use std::fmt::Formatter;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Name {
    Attribute(StringVar),
    Operator(Operator),
}

impl Name {
    pub fn do_each<T>(
        self,
        op: impl FnOnce(Operator) -> T,
        attr: impl FnOnce(StringVar) -> T,
    ) -> T {
        match self {
            Name::Attribute(s) => attr(s),
            Name::Operator(o) => op(o),
        }
    }

    pub fn do_each_ref<T>(
        &self,
        op: impl FnOnce(Operator) -> T,
        attr: impl FnOnce(&StringVar) -> T,
    ) -> T {
        match self {
            Name::Attribute(s) => attr(s),
            Name::Operator(o) => op(*o),
        }
    }

    pub fn as_str(&self) -> StringVar {
        match self {
            Name::Attribute(s) => s.clone(),
            Name::Operator(o) => o.name().into(),
        }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Name::Attribute(s) => s.fmt(f),
            Name::Operator(o) => o.name().fmt(f),
        }
    }
}
