use ascii::{AsciiChar, AsciiStr};
use downcast_rs::__std::slice::Chunks;
use std::iter::Peekable;
use std::str::{from_utf8_unchecked, CharIndices};

pub struct StrChunks<'a> {
    iterator: Peekable<CharIndices<'a>>,
    value: &'a str,
    count: usize,
}

pub struct AsciiChunks<'a> {
    value: Chunks<'a, AsciiChar>,
}

impl<'a> StrChunks<'a> {
    pub fn new(value: &str, count: usize) -> StrChunks {
        StrChunks {
            iterator: value.char_indices().peekable(),
            value,
            count,
        }
    }

    fn skip_count(&mut self) -> Option<usize> {
        for _ in 0..self.count - 1 {
            self.iterator.next()?;
        }
        self.iterator.peek().map(|x| x.0)
    }
}

impl<'a> AsciiChunks<'a> {
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
}