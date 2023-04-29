use std::{
    io::{Read, Write},
    marker::PhantomData,
};

use crate::{Error, LexOrd, LexOrdSer, Result};

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
}

impl<'a, R: Read, T: LexOrd> Iterator for ReadIter<'a, R, T> {
    type Item = Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        (|| {
            let mut first = [0];
            self.reader.read_exact(&mut first)?;
            match first[0] {
                0x00 => Ok(None),
                0x01 => {
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
    let mut iter_writer = IterWriter {
        writer,
        item_started: false,
    };
    for item in iter {
        item.to_write(&mut iter_writer)?;
        iter_writer.item_started = false;
    }
    writer.write_all(&[0x00])?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use std::fmt::Debug;

    use crate::LexOrd;

    pub fn test_format<T: LexOrd + Debug>(value: &T, buf: &[u8]) {
        let mut buf_from_value = vec![];
        value.to_write(&mut buf_from_value).unwrap();
        assert!(
            buf == buf_from_value,
            "{value:#?} -> {buf_from_value:x?} != {buf:x?}"
        );
        let value_from_buf = T::from_read(&mut buf_from_value.as_slice()).unwrap();
        assert!(
            value == &value_from_buf,
            "{buf:?} -> {value:?} != {value_from_buf:?}"
        );
    }

    pub fn test_write_read<T: LexOrd + Debug>(t: impl Iterator<Item = T>) {
        let mut last = None;
        let mut last_buf = vec![];
        for next in t {
            assert!(
                last.is_none() || last.as_ref().unwrap() < &next,
                "{last:?} >= {next:?}"
            );
            let mut buf = vec![];
            next.to_write(&mut buf).unwrap();
            assert!(
                last_buf < buf,
                "{last:?} >= {next:?} ({last_buf:#?} >= {buf:#?})"
            );
            let mut buf_slice = buf.as_slice();
            let next_reser = T::from_read(&mut buf_slice).unwrap();
            assert!(buf_slice.is_empty(), "({buf:?})");
            assert_eq!(next, next_reser, "({buf:?})");
            last = Some(next);
            last_buf = buf;
        }
    }
}
