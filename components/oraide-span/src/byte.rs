use std::{
    ops,
};

/// Byte index into a string
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteIndex(pub usize);

impl ByteIndex {
    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for ByteIndex {
    fn from(src: usize) -> ByteIndex {
        ByteIndex(src)
    }
}

impl ops::Add<ByteCount> for ByteIndex {
    type Output = ByteIndex;

    fn add(self, other: ByteCount) -> ByteIndex {
        ByteIndex::from(self.to_usize() + other.to_usize())
    }
}

impl ops::AddAssign<ByteCount> for ByteIndex {
    fn add_assign(&mut self, other: ByteCount) {
        self.0 += other.to_usize();
    }
}

impl ops::Sub<ByteIndex> for ByteIndex {
    type Output = ByteCount;

    fn sub(self, other: ByteIndex) -> ByteCount {
        ByteCount::from(self.to_usize() - other.to_usize())
    }
}

/// Byte count for a given string or character in some encoding
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteCount(pub usize);

impl ByteCount {
    /// ```rust
    /// # use oraide_span::ByteCount;
    /// let ch = 'ç';
    /// let byte_count = ByteCount::from_char_len_utf8(ch);
    /// 
    /// assert_eq!(byte_count, ByteCount(2));
    /// ```
    pub fn from_char_len_utf8(ch: char) -> ByteCount {
        ByteCount::from(ch.len_utf8())
    }

    /// ```rust
    /// # use oraide_span::ByteCount;
    /// let ch = 'ç';
    /// let byte_count = ByteCount::from_char_len_utf16(ch);
    /// 
    /// assert_eq!(byte_count, ByteCount(1));
    /// ```
    pub fn from_char_len_utf16(ch: char) -> ByteCount {
        ByteCount::from(ch.len_utf16())
    }

    /// ```rust
    /// # use oraide_span::ByteCount;
    /// let s = "true";
    /// let byte_count = ByteCount::from_str_len_utf8(s);
    /// 
    /// assert_eq!(byte_count, ByteCount(4));
    /// #
    /// # // exclude the remainder from the docs for the sake of
    /// # // brevity, but include them in the tests
    /// # assert_eq!(ByteCount::from_str_len_utf8("a"), ByteCount(1));
    /// # assert_eq!(ByteCount::from_str_len_utf8("å"), ByteCount(2));
    /// # assert_eq!(ByteCount::from_str_len_utf8("“"), ByteCount(3));
    /// ```
    pub fn from_str_len_utf8(s: &str) -> ByteCount {
        ByteCount::from(s.len())
    }

    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl ops::Add<ByteCount> for ByteCount {
    type Output = ByteCount;

    fn add(self, other: ByteCount) -> ByteCount {
        ByteCount::from(self.to_usize() + other.to_usize())
    }
}

impl ops::AddAssign<ByteCount> for ByteCount {
    fn add_assign(&mut self, other: ByteCount) {
        self.0 += other.to_usize();
    }
}

impl From<usize> for ByteCount {
    fn from(src: usize) -> ByteCount {
        ByteCount(src)
    }
}