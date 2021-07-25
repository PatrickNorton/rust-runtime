use crate::string_var::{MaybeAscii, StringVar};
use ascii::AsciiStr;
use std::iter::FusedIterator;
use std::ops::Add;

pub enum MixedIter<'a, T, U>
where
    T: Iterator<Item = &'a AsciiStr>,
    U: Iterator<Item = &'a str>,
{
    Ascii(T),
    Normal(U),
}

pub enum OwnedIter<'a, T, U, V, W>
where
    T: Iterator<Item = &'a AsciiStr>,
    U: Iterator<Item = &'static AsciiStr>,
    V: Iterator<Item = &'a str>,
    W: Iterator<Item = &'static str>,
{
    Ascii(T),
    AsciiLiteral(U),
    Normal(V),
    Literal(W),
}

impl<'a, T, U> Iterator for MixedIter<'a, T, U>
where
    T: Iterator<Item = &'a AsciiStr>,
    U: Iterator<Item = &'a str>,
{
    type Item = MaybeAscii<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MixedIter::Ascii(x) => x.next().map(MaybeAscii::Ascii),
            MixedIter::Normal(x) => x.next().map(MaybeAscii::Standard),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            MixedIter::Ascii(x) => x.size_hint(),
            MixedIter::Normal(x) => x.size_hint(),
        }
    }
}

impl<'a, T, U, V, W> Iterator for OwnedIter<'a, T, U, V, W>
where
    T: Iterator<Item = &'a AsciiStr>,
    U: Iterator<Item = &'static AsciiStr>,
    V: Iterator<Item = &'a str>,
    W: Iterator<Item = &'static str>,
{
    type Item = StringVar;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OwnedIter::Ascii(a) => a.next().map(|x| x.to_owned().into()),
            OwnedIter::AsciiLiteral(a) => a.next().map(Into::into),
            OwnedIter::Normal(s) => s.next().map(|x| x.to_owned().into()),
            OwnedIter::Literal(s) => s.next().map(Into::into),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            OwnedIter::Ascii(a) => a.size_hint(),
            OwnedIter::AsciiLiteral(a) => a.size_hint(),
            OwnedIter::Normal(s) => s.size_hint(),
            OwnedIter::Literal(s) => s.size_hint(),
        }
    }
}

impl<'a, T, U> DoubleEndedIterator for MixedIter<'a, T, U>
where
    T: DoubleEndedIterator + Iterator<Item = &'a AsciiStr>,
    U: DoubleEndedIterator + Iterator<Item = &'a str>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            MixedIter::Ascii(a) => a.next_back().map(MaybeAscii::Ascii),
            MixedIter::Normal(s) => s.next_back().map(MaybeAscii::Standard),
        }
    }
}

impl<'a, T, U, V, W> DoubleEndedIterator for OwnedIter<'a, T, U, V, W>
where
    T: DoubleEndedIterator + Iterator<Item = &'a AsciiStr>,
    U: DoubleEndedIterator + Iterator<Item = &'static AsciiStr>,
    V: DoubleEndedIterator + Iterator<Item = &'a str>,
    W: DoubleEndedIterator + Iterator<Item = &'static str>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            OwnedIter::Ascii(a) => a.next_back().map(|x| x.to_owned().into()),
            OwnedIter::AsciiLiteral(a) => a.next_back().map(Into::into),
            OwnedIter::Normal(s) => s.next_back().map(|x| x.to_owned().into()),
            OwnedIter::Literal(s) => s.next_back().map(Into::into),
        }
    }
}

impl<'a, T, U> ExactSizeIterator for MixedIter<'a, T, U>
where
    T: ExactSizeIterator + Iterator<Item = &'a AsciiStr>,
    U: ExactSizeIterator + Iterator<Item = &'a str>,
{
}

impl<'a, T, U, V, W> ExactSizeIterator for OwnedIter<'a, T, U, V, W>
where
    T: ExactSizeIterator + Iterator<Item = &'a AsciiStr>,
    U: ExactSizeIterator + Iterator<Item = &'static AsciiStr>,
    V: ExactSizeIterator + Iterator<Item = &'a str>,
    W: ExactSizeIterator + Iterator<Item = &'static str>,
{
}

impl<'a, T, U> FusedIterator for MixedIter<'a, T, U>
where
    T: FusedIterator + Iterator<Item = &'a AsciiStr>,
    U: FusedIterator + Iterator<Item = &'a str>,
{
}

impl<'a, T, U, V, W> FusedIterator for OwnedIter<'a, T, U, V, W>
where
    T: FusedIterator + Iterator<Item = &'a AsciiStr>,
    U: FusedIterator + Iterator<Item = &'static AsciiStr>,
    V: FusedIterator + Iterator<Item = &'a str>,
    W: FusedIterator + Iterator<Item = &'static str>,
{
}

impl Add for &StringVar {
    type Output = StringVar;

    fn add(self, rhs: Self) -> Self::Output {
        (self.as_owned() + rhs).into()
    }
}

#[cfg(test)]
mod test {
    use crate::string_var::StringVar;

    #[test]
    fn string_add() {
        let a = StringVar::from("abc");
        let b = StringVar::from("def");
        assert_eq!(&*(&a + &b), "abcdef");
        drop(a);
        drop(b);
    }
}
