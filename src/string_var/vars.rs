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

#[cfg(test)]
mod test {
    use crate::string_var::{AsciiVar, StrVar};
    use ascii::AsAsciiStr;

    #[test]
    fn str_deref() {
        let a = StrVar::Literal("abc");
        assert_eq!(&*a, "abc");
    }

    #[test]
    fn ascii_deref() {
        let a = AsciiVar::Literal("abc".as_ascii_str().unwrap());
        assert_eq!(&*a, "abc");
    }
}
