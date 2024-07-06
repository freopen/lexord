use std::{
    io::{Read, Write},
    marker::PhantomData,
};

use crate::{Error, LexOrd, LexOrdSer, ObjectType, PrefixRead, Result};

pub struct ReadIter<'a, R: Read, T: LexOrd> {
    reader: &'a mut PrefixRead<R>,
    zero_sized_count: Option<usize>,
    _phantom: PhantomData<T>,
}

impl<'a, R: Read, T: LexOrd> ReadIter<'a, R, T> {
    pub fn new(reader: &'a mut PrefixRead<R>) -> ReadIter<'a, R, T> {
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
            if T::object_type() == ObjectType::ZeroSized {
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
                match (first[0], T::object_type()) {
                    (0x00, _) => Ok(None),
                    (0x01, ObjectType::Default) => {
                        let mut second = [0];
                        self.reader.read_exact(&mut second)?;
                        if second[0] > 0x01 {
                            return Err(Error::Parse(
                                "Iterator element can't start with 0x01{0x02+}".to_string(),
                            ));
                        }
                        self.reader.push(second[0]);
                        Ok(Some(T::from_read(self.reader)?))
                    }
                    _ => {
                        self.reader.push(first[0]);
                        Ok(Some(T::from_read(self.reader)?))
                    }
                }
            }
        })()
        .transpose()
    }
}

pub fn write_iterator<'a, T: LexOrdSer + 'a>(
    writer: &mut impl Write,
    iter: &mut impl Iterator<Item = &'a T>,
) -> Result {
    match T::object_type() {
        ObjectType::Default => {
            for item in iter {
                let mut ser = vec![];
                item.to_write(&mut ser)?;
                match ser.first() {
                    None => {
                        return Err(Error::Internal(
                            "Empty encoding for an item with default object type".to_string(),
                        ))
                    }
                    Some(0x00 | 0x01) => writer.write_all(&[0x01])?,
                    _ => (),
                }
                writer.write_all(&ser)?;
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
