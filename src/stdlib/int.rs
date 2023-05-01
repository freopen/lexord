use std::io::{Read, Write};

use crate::{LexOrd, LexOrdSer, Result};

macro_rules! lexord_uint {
    ($t:ty) => {
        impl LexOrd for $t {
            fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
                let mut buf = [0u8; std::mem::size_of::<$t>()];
                reader.read_exact(&mut buf)?;
                Ok(<$t>::from_be_bytes(buf))
            }
        }
        impl LexOrdSer for $t {
            fn to_write<W: Write>(&self, writer: &mut W) -> Result {
                writer.write_all(&self.to_be_bytes())?;
                Ok(())
            }
        }
    };
}

lexord_uint!(u8);
lexord_uint!(u16);
lexord_uint!(u32);
lexord_uint!(u64);
lexord_uint!(u128);
lexord_uint!(usize);

macro_rules! lexord_int {
    ($t:ty) => {
        impl LexOrd for $t {
            fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
                let mut buf = [0u8; std::mem::size_of::<$t>()];
                reader.read_exact(&mut buf)?;
                Ok(<$t>::from_be_bytes(buf) ^ <$t>::MIN)
            }
        }
        impl LexOrdSer for $t {
            fn to_write<W: Write>(&self, writer: &mut W) -> Result {
                writer.write_all(&(self ^ <$t>::MIN).to_be_bytes())?;
                Ok(())
            }
        }
    };
}

lexord_int!(i8);
lexord_int!(i16);
lexord_int!(i32);
lexord_int!(i64);
lexord_int!(i128);
lexord_int!(isize);

#[cfg(test)]
mod tests {
    use crate::util::test::{test_format, test_write_read};

    #[test]
    fn test_lexord_uint() {
        test_format(&1u32, &[0x00, 0x00, 0x00, 0x01]);
        test_write_read(u16::MIN..=u16::MAX);
    }
    #[test]
    fn test_lexord_int() {
        test_format(&-1, &[0x7F, 0xFF, 0xFF, 0xFF]);
        test_format(&0, &[0x80, 0x00, 0x00, 0x00]);
        test_format(&1, &[0x80, 0x00, 0x00, 0x01]);
        test_format(&i32::MIN, &[0x00, 0x00, 0x00, 0x00]);
        test_format(&i32::MAX, &[0xFF, 0xFF, 0xFF, 0xFF]);
        test_write_read(i16::MIN..=i16::MAX);
    }
}
