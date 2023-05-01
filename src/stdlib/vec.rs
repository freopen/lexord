use std::io::{Read, Write};

use crate::{
    util::iterator::{write_iterator, ReadIter},
    LexOrd, LexOrdSer, Result,
};

impl<T: LexOrd> LexOrd for Vec<T> {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
        ReadIter::new(reader).collect()
    }
}

impl<T: LexOrdSer> LexOrdSer for &[T] {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        write_iterator(writer, &mut self.iter())
    }
}

impl<T: LexOrdSer, const N: usize> LexOrdSer for &[T; N] {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        write_iterator(writer, &mut self.iter())
    }
}

impl<T: LexOrdSer> LexOrdSer for Vec<T> {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        self.as_slice().to_write(writer)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::test::{test_format, test_write_read};

    #[test]
    fn test_vec_format() {
        test_format(&Vec::<u16>::new(), &[0x00]);
        test_format(&vec![0u16], &[0x01, 0x00, 0x00, 0x00]);
        test_format(&vec![1u16], &[0x01, 0x00, 0x01, 0x00]);
        test_format(&vec![0x0100u16], &[0x01, 0x01, 0x00, 0x00]);
        test_format(&vec![0x0200u16], &[0x02, 0x00, 0x00]);
        test_format(
            &vec![(), (), ()],
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03],
        );
    }

    #[test]
    fn test_order() {
        let mut values = vec![];
        values.push(vec![]);
        values.extend((i8::MIN..=i8::MAX).map(|x| vec![x]));
        values
            .extend((i8::MIN..=i8::MAX).flat_map(|x| (i8::MIN..=i8::MAX).map(move |y| vec![x, y])));
        values.sort();
        test_write_read(values.into_iter());
    }
}
