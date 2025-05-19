use std::str::Chars;

/// Provides the characters of a str and remembers
/// your byte position in the str.
pub struct ByteChars<'a> {
    inner: Chars<'a>,
    bytes: usize,
}

impl<'a> ByteChars<'a> {
    pub fn new(inner: &'a str) -> Self {
        Self {
            inner: inner.chars(),
            bytes: 0,
        }
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

impl<'a> Iterator for ByteChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.inner.next();
        if let Some(r) = &ret {
            self.bytes += r.len_utf8();
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use crate::byte_chars::ByteChars;

    #[test]
    fn empty_str_gives_empty_iter() {
        assert_eq!(ByteChars::new("").collect::<Vec<_>>(), &[])
    }

    #[test]
    fn iterating_over_byte_chars_gives_all_chars() {
        assert_eq!(ByteChars::new("abc").collect::<Vec<_>>(), &['a', 'b', 'c'])
    }

    #[test]
    fn counting_one_byte_chars_gives_correct_answer() {
        let mut ch = ByteChars::new("abc");
        assert_eq!(ch.bytes(), 0);
        ch.next();
        assert_eq!(ch.bytes(), 1);
        ch.next();
        assert_eq!(ch.bytes(), 2);
        ch.next();
        assert_eq!(ch.bytes(), 3);
        assert!(ch.next().is_none());
        assert_eq!(ch.bytes(), 3);
    }

    #[test]
    fn counting_multi_byte_chars_gives_correct_answer() {
        let mut ch = ByteChars::new("a\u{1f4a6}c");
        assert_eq!(ch.bytes(), 0);
        ch.next();
        assert_eq!(ch.bytes(), 1);
        ch.next();
        assert_eq!(ch.bytes(), 5, "Pile of poo is 4");
        ch.next();
        assert_eq!(ch.bytes(), 6);
    }
}
