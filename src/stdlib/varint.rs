use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
};

use crate::{Error, LexOrd, LexOrdSer, Result};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VarInt<T: Ord>(T);

impl<T: Ord> Deref for VarInt<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Ord> DerefMut for VarInt<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Ord> From<T> for VarInt<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

macro_rules! lexord_varint_uint {
    ($t:ty) => {
        impl LexOrdSer for VarInt<$t> {
            fn to_write<W: Write>(&self, writer: &mut W) -> Result {
                match self.0 as u128 {
                    0..=0x3F => (self.0 as u8 | 0x80).to_write(writer)?,
                    0x40..=0x1FFF => (self.0 as u16 | 0xC000).to_write(writer)?,
                    0x2000..=0x0FFFFFFF => (self.0 as u32 | 0xE0000000).to_write(writer)?,
                    0x10000000..=0x07FFFFFFFFFFFFFF => {
                        (self.0 as u64 | 0xF000000000000000).to_write(writer)?
                    }
                    _ => {
                        writer.write_all(&[0xF8])?;
                        (self.0 as u128).to_write(writer)?;
                    }
                }
                Ok(())
            }
        }
        impl LexOrd for VarInt<$t> {
            fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
                let mut first = [0u8; 1];
                reader.read_exact(&mut first)?;
                match first[0] {
                    0x80..=0xBF => Ok(Self((first[0] & !0x80) as $t)),
                    0xC0..=0xDF => {
                        let mut buf = [first[0] & !0xC0, 0];
                        reader.read_exact(&mut buf[1..])?;
                        Ok(VarInt(u16::from_be_bytes(buf).try_into()?))
                    }
                    0xE0..=0xEF => {
                        let mut buf = [first[0] & !0xE0, 0, 0, 0];
                        reader.read_exact(&mut buf[1..])?;
                        Ok(VarInt(u32::from_be_bytes(buf).try_into()?))
                    }
                    0xF0..=0xF7 => {
                        let mut buf = [first[0] & !0xF0, 0, 0, 0, 0, 0, 0, 0];
                        reader.read_exact(&mut buf[1..])?;
                        Ok(VarInt(u64::from_be_bytes(buf).try_into()?))
                    }
                    0xF8 => Ok(VarInt(u128::from_read(reader)?.try_into()?)),
                    _ => Err(Error::Parse(format!(
                        "Unsupported VarInt prefix: {first:?}"
                    ))),
                }
            }
        }
    };
}

lexord_varint_uint!(u8);
lexord_varint_uint!(u16);
lexord_varint_uint!(u32);
lexord_varint_uint!(u64);
lexord_varint_uint!(u128);
lexord_varint_uint!(usize);

impl LexOrdSer for usize {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        VarInt::to_write(&VarInt(*self), writer)
    }
}
impl LexOrd for usize {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
        VarInt::from_read(reader).map(|v| v.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        stdlib::varint::VarInt,
        util::test::{test_format, test_write_read},
    };

    #[test]
    fn test_varint_uint_format() {
        test_format(&VarInt(1u8), &[0x81]);
        test_format(&VarInt(1u16), &[0x81]);
        test_format(&VarInt(128u8), &[0xc0, 0x80]);
        test_format(&VarInt(128u16), &[0xc0, 0x80]);
        test_format(
            &VarInt(u32::MAX),
            &[0xf0, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF],
        );
        test_format(
            &VarInt(u128::MAX),
            &[
                0xf8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF,
            ],
        );
    }

    #[test]
    fn test_varint_u16_all() {
        test_write_read(u16::MIN..=u16::MAX);
    }
}
