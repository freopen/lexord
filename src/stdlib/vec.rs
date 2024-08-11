use std::io::{Read, Write};

use crate::{
    util::iterator::{write_iterator, write_seq_iterator, ReadIter},
    LexOrd, LexOrdSer, Result,
};

impl<T: LexOrdSer> LexOrdSer for [T] {
    fn to_write(&self, writer: &mut impl Write) -> Result {
        write_iterator(writer, &mut self.iter())
    }
    fn to_write_seq(&self, writer: &mut impl Write) -> Result {
        write_seq_iterator(writer, &mut self.iter())
    }
}

impl<T: LexOrdSer, const N: usize> LexOrdSer for [T; N] {
    fn to_write(&self, writer: &mut impl Write) -> Result {
        write_iterator(writer, &mut self.iter())
    }
    fn to_write_seq(&self, writer: &mut impl Write) -> Result {
        write_seq_iterator(writer, &mut self.iter())
    }
}

impl<T: LexOrdSer> LexOrdSer for Vec<T> {
    fn to_write(&self, writer: &mut impl Write) -> Result {
        self.as_slice().to_write(writer)
    }
    fn to_write_seq(&self, writer: &mut impl Write) -> Result {
        write_seq_iterator(writer, &mut self.iter())
    }
}

impl<T: LexOrd> LexOrd for Vec<T> {
    fn from_read(reader: &mut impl Read) -> Result<Self> {
        ReadIter::new(reader).collect()
    }
    fn from_read_seq(first: u8, reader: &mut impl Read) -> Result<Self> {
        ReadIter::new_seq(first, reader)?.collect()
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::util::test::encode;

    #[test]
    fn test_vec_format() {
        assert_snapshot!(encode(<Vec<()>>::new()), @"00");
        assert_snapshot!(encode(<Vec<u8>>::new()), @"00");
        assert_snapshot!(encode(<Vec<u16>>::new()), @"00");
        assert_snapshot!(encode(vec![()]), @"01 00");
        assert_snapshot!(encode(vec![0u8, 1u8, 2u8, 3u8]), @"01 00 01 01 02 03 00");
        assert_snapshot!(encode(vec![0u16, 1u16, 2u16, 3u16]), @"80 81 82 83 00");
    }
}
