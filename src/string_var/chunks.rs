use ascii::{AsciiChar, AsciiStr};
use std::iter::FusedIterator;
use std::iter::Peekable;
use std::slice::Chunks;
use std::str::{from_utf8_unchecked, CharIndices};

pub struct StrChunks<'a> {
    iterator: Peekable<CharIndices<'a>>,
    value: &'a str,
    count: usize,
}

pub struct AsciiChunks<'a> {
    value: Chunks<'a, AsciiChar>,
}

impl StrChunks<'_> {
    pub fn new(value: &str, count: usize) -> StrChunks {
        StrChunks {
            iterator: value.char_indices().peekable(),
            value,
            count,
        }
    }

    fn skip_count(&mut self) -> Option<usize> {
        // n-2 because n-1 will give out n values, and one is already used to get start
        self.iterator.nth(self.count - 2)?;
        self.iterator.peek().map(|x| x.0)
    }
}

impl AsciiChunks<'_> {
    pub fn new(value: &AsciiStr, count: usize) -> AsciiChunks {
        AsciiChunks {
            value: value.as_slice().chunks(count),
        }
    }
}

impl<'a> Iterator for StrChunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.count > 0);
        let start = self.iterator.next()?.0;
        // SAFETY: str_indices are guaranteed to point to valid UTF-8 indices, and indexing from
        //         valid indices will always produce a valid string. Because this is in a loop,
        //         using the checked version will be *very* slow
        unsafe {
            let slice = match self.skip_count() {
                Option::None => &self.value.as_bytes()[start..],
                Option::Some(stop) => &self.value.as_bytes()[start..stop],
            };
            Option::Some(from_utf8_unchecked(slice))
        }
    }
}

impl<'a> Iterator for AsciiChunks<'a> {
    type Item = &'a AsciiStr;

    fn next(&mut self) -> Option<Self::Item> {
        self.value.next().map(Into::into)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.value.size_hint()
    }
}

impl DoubleEndedIterator for AsciiChunks<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.value.next_back().map(Into::into)
    }
}

impl ExactSizeIterator for AsciiChunks<'_> {}

impl FusedIterator for AsciiChunks<'_> {}
