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
    use crate::helpers::tests::test_write_read;

    use super::*;

    #[test]
    fn test_lexord_uint() -> Result {
        test_write_read(u16::MIN..=u16::MAX);
        Ok(())
    }
    #[test]
    fn test_lexord_int() -> Result {
        test_write_read(i16::MIN..=i16::MAX);
        Ok(())
    }
}
