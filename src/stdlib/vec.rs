use std::io::{Read, Write};

use crate::{
    util::iterator::{write_iterator, ReadIter},
    LexOrd, LexOrdSer, PrefixRead, Result,
};

impl<T: LexOrdSer> LexOrdSer for [T] {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        write_iterator(writer, &mut self.iter())
    }
}

impl<T: LexOrdSer, const N: usize> LexOrdSer for [T; N] {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        write_iterator(writer, &mut self.iter())
    }
}

impl<T: LexOrdSer> LexOrdSer for Vec<T> {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        self.as_slice().to_write(writer)
    }
}

impl<T: LexOrd> LexOrd for Vec<T> {
    fn from_read<R: Read>(reader: &mut PrefixRead<R>) -> Result<Self> {
        ReadIter::new(reader).collect()
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::util::test::encode;

    #[test]
    fn test_vec_format() {
        assert_snapshot!(encode(<Vec<()>>::new()), @"80");
        assert_snapshot!(encode(<Vec<u8>>::new()), @"00");
        assert_snapshot!(encode(<Vec<u16>>::new()), @"00");
        assert_snapshot!(encode(vec![()]), @"81");
        assert_snapshot!(encode(vec![0u8, 1u8, 2u8, 3u8]), @"01 00 01 01 02 03 00");
        assert_snapshot!(encode(vec![0u16, 1u16, 2u16, 3u16]), @"80 81 82 83 00");
    }
}
