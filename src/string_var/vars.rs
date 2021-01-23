use ascii::AsciiStr;
use std::ops::Deref;
use std::sync::Arc;

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
