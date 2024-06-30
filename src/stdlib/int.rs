use std::io::{Read, Write};

use crate::{Error, LexOrd, LexOrdSer, ObjectType, Result};

impl LexOrdSer for bool {
    fn object_type() -> ObjectType {
        ObjectType::CantStartWithZero
    }

    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        writer.write_all(&[*self as u8 + 0x80])?;
        Ok(())
    }
}

impl LexOrd for bool {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0] != 0x80)
    }
}

impl LexOrdSer for u8 {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        writer.write_all(&[*self])?;
        Ok(())
    }
}

impl LexOrd for u8 {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl LexOrdSer for i8 {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        writer.write_all(&[(self ^ i8::MIN) as u8])?;
        Ok(())
    }
}

impl LexOrd for i8 {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0] as i8 ^ i8::MIN)
    }
}

macro_rules! lexord_uint {
    ($t:ty) => {
        impl LexOrdSer for $t {
            fn object_type() -> ObjectType {
                ObjectType::CantStartWithZero
            }

            fn to_write<W: Write>(&self, writer: &mut W) -> Result {
                match *self as u128 {
                    0..=0x3F => writer.write_all(&[*self as u8 | 0x80])?,
                    0x40..=0x1FFF => writer.write_all(&(*self as u16 | 0xC000).to_be_bytes())?,
                    0x2000..=0x0FFFFFFF => {
                        writer.write_all(&(*self as u32 | 0xE0000000).to_be_bytes())?
                    }
                    0x10000000..=0x07FFFFFFFFFFFFFF => {
                        writer.write_all(&(*self as u64 | 0xF000000000000000).to_be_bytes())?
                    }
                    _ => {
                        writer.write_all(&[0xF8])?;
                        writer.write_all(&(*self as u128).to_be_bytes())?
                    }
                }
                Ok(())
            }
        }
        impl LexOrd for $t {
            fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf[..1])?;
                match buf[0] {
                    0x80..=0xBF => Ok((buf[0] & !0x80) as $t),
                    0xC0..=0xDF => {
                        buf[0] &= !0xC0;
                        reader.read_exact(&mut buf[1..2])?;
                        Ok(u16::from_be_bytes(buf[..2].try_into().unwrap()).try_into()?)
                    }
                    0xE0..=0xEF => {
                        buf[0] &= !0xE0;
                        reader.read_exact(&mut buf[1..4])?;
                        Ok(u32::from_be_bytes(buf[..4].try_into().unwrap()).try_into()?)
                    }
                    0xF0..=0xF7 => {
                        buf[0] &= !0xF0;
                        reader.read_exact(&mut buf[1..8])?;
                        Ok(u64::from_be_bytes(buf[..8].try_into().unwrap()).try_into()?)
                    }
                    0xF8 => {
                        reader.read_exact(&mut buf)?;
                        Ok(u128::from_be_bytes(buf).try_into()?)
                    }
                    _ => Err(Error::Parse(format!("Unsupported VarInt prefix: {buf:?}"))),
                }
            }
        }
    };
}

lexord_uint!(u16);
lexord_uint!(u32);
lexord_uint!(u64);
lexord_uint!(u128);
lexord_uint!(usize);

macro_rules! lexord_int {
    ($t:ty) => {
        impl LexOrdSer for $t {
            fn object_type() -> ObjectType {
                ObjectType::CantStartWithZero
            }

            fn to_write<W: Write>(&self, writer: &mut W) -> Result {
                match *self as i128 {
                    0..=i128::MAX => (*self as u128).to_write(writer)?,
                    -0x40..=-0x01 => writer.write_all(&(*self as i8 & 0x7F).to_be_bytes())?,
                    -0x2000..=-0x41 => writer.write_all(&(*self as i16 & 0x3FFF).to_be_bytes())?,
                    -0x10000000..=-0x2001 => {
                        writer.write_all(&(*self as i32 & 0x1FFFFFFF).to_be_bytes())?
                    }
                    -0x0800000000000000..=-0x01000001 => {
                        writer.write_all(&(*self as i64 & 0x0FFFFFFFFFFFFFFF).to_be_bytes())?
                    }
                    i128::MIN..=-0x0800000000000001 => {
                        writer.write_all(&[0x04])?;
                        writer.write_all(&(*self as i128).to_be_bytes())?
                    }
                }
                Ok(())
            }
        }
        impl LexOrd for $t {
            fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf[..1])?;
                match buf[0] {
                    0x80..=0xBF => Ok((buf[0] & !0x80) as $t),
                    0xC0..=0xDF => {
                        buf[0] &= !0xC0;
                        reader.read_exact(&mut buf[1..2])?;
                        Ok(u16::from_be_bytes(buf[..2].try_into().unwrap()).try_into()?)
                    }
                    0xE0..=0xEF => {
                        buf[0] &= !0xE0;
                        reader.read_exact(&mut buf[1..4])?;
                        Ok(u32::from_be_bytes(buf[..4].try_into().unwrap()).try_into()?)
                    }
                    0xF0..=0xF7 => {
                        buf[0] &= !0xF0;
                        reader.read_exact(&mut buf[1..8])?;
                        Ok(u64::from_be_bytes(buf[..8].try_into().unwrap()).try_into()?)
                    }
                    0xF8 => {
                        reader.read_exact(&mut buf)?;
                        Ok(u128::from_be_bytes(buf).try_into()?)
                    }
                    0x40..=0x7F => Ok((buf[0] | 0x80) as i8 as $t),
                    0x20..=0x3F => {
                        buf[0] |= 0xC0;
                        reader.read_exact(&mut buf[1..2])?;
                        Ok(i16::from_be_bytes(buf[..2].try_into().unwrap()).try_into()?)
                    }
                    0x10..=0x1F => {
                        buf[0] |= 0xE0;
                        reader.read_exact(&mut buf[1..4])?;
                        Ok(i32::from_be_bytes(buf[..4].try_into().unwrap()).try_into()?)
                    }
                    0x08..=0x0F => {
                        buf[0] |= 0xF0;
                        reader.read_exact(&mut buf[1..8])?;
                        Ok(i64::from_be_bytes(buf[..8].try_into().unwrap()).try_into()?)
                    }
                    0x04 => {
                        reader.read_exact(&mut buf)?;
                        Ok(i128::from_be_bytes(buf).try_into()?)
                    }
                    _ => Err(Error::Parse(format!("Unsupported VarInt prefix: {buf:?}"))),
                }
            }
        }
    };
}

lexord_int!(i16);
lexord_int!(i32);
lexord_int!(i64);
lexord_int!(i128);
lexord_int!(isize);

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::util::test::encode;

    #[test]
    fn test_u8() {
        assert_snapshot!(encode(0u8), @"00");
        assert_snapshot!(encode(1u8), @"01");
        assert_snapshot!(encode(10u8), @"0A");
        assert_snapshot!(encode(254u8), @"FE");
        assert_snapshot!(encode(255u8), @"FF");
    }

    #[test]
    fn test_i8() {
        assert_snapshot!(encode(0i8), @"80");
        assert_snapshot!(encode(1i8), @"81");
        assert_snapshot!(encode(10i8), @"8A");
        assert_snapshot!(encode(126i8), @"FE");
        assert_snapshot!(encode(127i8), @"FF");
        assert_snapshot!(encode(-128i8), @"00");
        assert_snapshot!(encode(-127i8), @"01");
    }

    macro_rules! gen_encode_varint {
        ($($t:ty)+) => {
            fn encode_varint<T: Copy $(+ TryInto<$t>)*>(value: T) -> String {
                let mut buf = vec![];
                $(
                    if let Ok(typed) = TryInto::<$t>::try_into(value) {
                        buf.push(encode(typed));
                    }
                )*
                buf.dedup();
                assert_eq!(buf.len(), 1, "{buf:?}");
                buf.pop().unwrap()
            }

        };
    }
    gen_encode_varint!(u16 u32 u64 u128 usize i16 i32 i64 i128 isize);

    #[test]
    fn test_varint() {
        assert_snapshot!(encode_varint(0), @"80");
        assert_snapshot!(encode_varint(1), @"81");
        assert_snapshot!(encode_varint(-1), @"7F");
        assert_snapshot!(encode_varint(u8::MAX), @"C0 FF");
        assert_snapshot!(encode_varint(u16::MAX), @"E0 00 FF FF");
        assert_snapshot!(encode_varint(u32::MAX), @"F0 00 00 00 FF FF FF FF");
        assert_snapshot!(encode_varint(u64::MAX),
                         @"F8 00 00 00 00 00 00 00 00 FF FF FF FF FF FF FF FF");
        assert_snapshot!(encode_varint(u128::MAX),
                         @"F8 FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF");
        assert_snapshot!(encode_varint(i8::MAX), @"C0 7F");
        assert_snapshot!(encode_varint(i16::MAX), @"E0 00 7F FF");
        assert_snapshot!(encode_varint(i32::MAX), @"F0 00 00 00 7F FF FF FF");
        assert_snapshot!(encode_varint(i64::MAX),
                         @"F8 00 00 00 00 00 00 00 00 7F FF FF FF FF FF FF FF");
        assert_snapshot!(encode_varint(i128::MAX),
                         @"F8 7F FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF");
        assert_snapshot!(encode_varint(i8::MIN), @"3F 80");
        assert_snapshot!(encode_varint(i16::MIN), @"1F FF 80 00");
        assert_snapshot!(encode_varint(i32::MIN), @"0F FF FF FF 80 00 00 00");
        assert_snapshot!(encode_varint(i64::MIN),
                         @"04 FF FF FF FF FF FF FF FF 80 00 00 00 00 00 00 00");
        assert_snapshot!(encode_varint(i128::MIN),
                         @"04 80 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00");
    }
}
