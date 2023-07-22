use std::{
    io::{Read, Write},
    marker::PhantomData,
};

use crate::{Error, LexOrd, LexOrdSer, ObjectType, Result};

pub struct ReadIter<'a, R: Read, T: LexOrd> {
    reader: &'a mut R,
    zero_sized_count: Option<usize>,
    _phantom: PhantomData<T>,
}

impl<'a, R: Read, T: LexOrd> ReadIter<'a, R, T> {
    pub fn new(reader: &'a mut R) -> ReadIter<'a, R, T> {
        ReadIter {
            reader,
            zero_sized_count: None,
            _phantom: PhantomData,
        }
    }
}

impl<'a, R: Read, T: LexOrd> Iterator for ReadIter<'a, R, T> {
    type Item = Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        (|| {
            if let ObjectType::ZeroSized = T::OBJECT_TYPE {
                if self.zero_sized_count.is_none() {
                    self.zero_sized_count = Some(usize::from_read(self.reader)?);
                }
                if self.zero_sized_count.unwrap() == 0 {
                    Ok(None)
                } else {
                    self.zero_sized_count = Some(self.zero_sized_count.unwrap() - 1);
                    Ok(Some(T::from_read(self.reader)?))
                }
            } else {
                let mut first = [0];
                self.reader.read_exact(&mut first)?;
                match (first[0], T::OBJECT_TYPE) {
                    (0x00, _) => Ok(None),
                    (0x01, ObjectType::Default) => {
                        let mut second = [0];
                        self.reader.read_exact(&mut second)?;
                        if second[0] > 0x01 {
                            return Err(Error::Parse(
                                "Iterator element can't start with 0x01{0x02+}".to_string(),
                            ));
                        }
                        let mut reader = second.chain(self.reader.by_ref());
                        Ok(Some(T::from_read(&mut reader)?))
                    }
                    _ => {
                        let mut reader = first.chain(self.reader.by_ref());
                        Ok(Some(T::from_read(&mut reader)?))
                    }
                }
            }
        })()
        .transpose()
    }
}

struct IterWriter<'a, W: Write> {
    writer: &'a mut W,
    item_started: bool,
}

impl<'a, W: Write> Write for IterWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.item_started {
            self.writer.write(buf)
        } else {
            self.item_started = true;
            match buf.first() {
                None => return Ok(0),
                Some(0x00 | 0x01) => self.writer.write_all(&[0x01])?,
                _ => (),
            }
            self.writer.write(buf)
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

pub fn write_iterator<'a, T: LexOrdSer + 'a>(
    writer: &mut impl Write,
    iter: &mut impl Iterator<Item = &'a T>,
) -> Result {
    match T::OBJECT_TYPE {
        ObjectType::Default => {
            let mut iter_writer = IterWriter {
                writer,
                item_started: false,
            };
            for item in iter {
                item.to_write(&mut iter_writer)?;
                iter_writer.item_started = false;
            }
            writer.write_all(&[0x00])?;
        }
        ObjectType::CantStartWithZero => {
            for item in iter {
                item.to_write(writer)?;
            }
            writer.write_all(&[0x00])?;
        }
        ObjectType::ZeroSized => {
            iter.count().to_write(writer)?;
        }
    }
    Ok(())
}
