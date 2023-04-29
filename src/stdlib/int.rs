use std::io::{Read, Write};

use crate::{LexOrd, Result};

macro_rules! lexord_uint {
    ($t:ty) => {
        impl LexOrd for $t {
            fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
                let mut buf = [0u8; std::mem::size_of::<$t>()];
                reader.read_exact(&mut buf)?;
                Ok(<$t>::from_be_bytes(buf))
            }
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
    use super::*;

    #[test]
    fn test_lexord_uint() -> Result {
        let mut buf = vec![];
        0x01020304u32.to_write(&mut buf)?;
        assert_eq!(buf, vec![0x01, 0x02, 0x03, 0x04]);
        assert_eq!(u32::from_read(&mut buf.as_slice())?, 0x01020304);
        Ok(())
    }
    #[test]
    fn test_lexord_int() -> Result {
        let mut buf = vec![];
        0x01020304i32.to_write(&mut buf)?;
        assert_eq!(buf, vec![0x81, 0x02, 0x03, 0x04]);
        assert_eq!(i32::from_read(&mut buf.as_slice())?, 0x01020304);
        Ok(())
    }
}
