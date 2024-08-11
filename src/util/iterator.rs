use std::{
    io::{Read, Write},
    marker::PhantomData,
};

use crate::{LexOrd, LexOrdSer, Result};

pub struct ReadIter<'a, R: Read, T: LexOrd> {
    reader: &'a mut R,
    _phantom: PhantomData<T>,
}

impl<'a, R: Read, T: LexOrd> ReadIter<'a, R, T> {
    pub fn new(reader: &'a mut R) -> ReadIter<'a, R, T> {
        ReadIter {
            reader,
            _phantom: PhantomData,
        }
    }
    pub fn new_seq(first: u8, reader: &'a mut R) -> Result<ReadIter<'a, R, T>> {
        debug_assert_eq!(first, 0x01);
        Ok(ReadIter {
            reader,
            _phantom: PhantomData,
        })
    }
}

impl<'a, R: Read, T: LexOrd> Iterator for ReadIter<'a, R, T> {
    type Item = Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut first = [0];
        if let Err(error) = self.reader.read_exact(&mut first) {
            return Some(Err(error.into()));
        }
        match first[0] {
            0x00 => None,
            first => Some(T::from_read_seq(first, self.reader)),
        }
    }
}

pub fn write_iterator<'a, T: LexOrdSer + 'a>(
    writer: &mut impl Write,
    iter: &mut impl Iterator<Item = &'a T>,
) -> Result {
    for item in iter {
        item.to_write_seq(writer)?;
    }
    writer.write_all(&[0x00])?;
    Ok(())
}

pub fn write_seq_iterator<'a, T: LexOrdSer + 'a>(
    writer: &mut impl Write,
    iter: &mut impl Iterator<Item = &'a T>,
) -> Result {
    writer.write_all(&[0x01])?;
    write_iterator(writer, iter)
}
