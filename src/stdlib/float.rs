use std::io::{Read, Write};

use crate::{LexOrd, LexOrdSer, Result};

impl LexOrdSer for f32 {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        if self == &0.0 {
            writer.write_all(&[0x80, 0x00, 0x00, 0x00])?;
            return Ok(());
        }
        let mut bits = self.to_bits();
        bits ^= 0x80000000 | (((bits as i32) >> 31) as u32);
        writer.write_all(&bits.to_be_bytes())?;
        Ok(())
    }
}

impl LexOrd for f32 {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let mut bits = u32::from_be_bytes(buf);
        bits ^= 0x80000000 | ((((!bits) as i32) >> 31) as u32);
        Ok(f32::from_bits(bits))
    }
}

impl LexOrdSer for f64 {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        if self == &0.0 {
            writer.write_all(&[0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            return Ok(());
        }
        let mut bits = self.to_bits();
        bits ^= 0x8000000000000000 | (((bits as i64) >> 63) as u64);
        writer.write_all(&bits.to_be_bytes())?;
        Ok(())
    }
}

impl LexOrd for f64 {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        let mut bits = u64::from_be_bytes(buf);
        bits ^= 0x8000000000000000 | ((((!bits) as i64) >> 63) as u64);
        Ok(f64::from_bits(bits))
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::util::test::encode;

    #[test]
    fn test_f32() {
        assert_snapshot!(encode(0f32), @"80 00 00 00");
        assert_snapshot!(encode(-0f32), @"80 00 00 00");
        assert_snapshot!(encode(1f32), @"BF 80 00 00");
        assert_snapshot!(encode(-1f32), @"40 7F FF FF");
        assert_snapshot!(encode(f32::NAN), @"FF C0 00 00");
        assert_snapshot!(encode(f32::INFINITY), @"FF 80 00 00");
        assert_snapshot!(encode(f32::NEG_INFINITY), @"00 7F FF FF");
        assert_snapshot!(encode(f32::EPSILON), @"B4 00 00 00");
        assert_snapshot!(encode(f32::MAX), @"FF 7F FF FF");
        assert_snapshot!(encode(f32::MIN), @"00 80 00 00");
        assert_snapshot!(encode(f32::MIN_POSITIVE), @"80 80 00 00");
    }

    #[test]
    fn test_f64() {
        assert_snapshot!(encode(0f64), @"80 00 00 00 00 00 00 00");
        assert_snapshot!(encode(-0f64), @"80 00 00 00 00 00 00 00");
        assert_snapshot!(encode(1f64), @"BF F0 00 00 00 00 00 00");
        assert_snapshot!(encode(-1f64), @"40 0F FF FF FF FF FF FF");
        assert_snapshot!(encode(f64::NAN), @"FF F8 00 00 00 00 00 00");
        assert_snapshot!(encode(f64::INFINITY), @"FF F0 00 00 00 00 00 00");
        assert_snapshot!(encode(f64::NEG_INFINITY), @"00 0F FF FF FF FF FF FF");
        assert_snapshot!(encode(f64::EPSILON), @"BC B0 00 00 00 00 00 00");
        assert_snapshot!(encode(f64::MAX), @"FF EF FF FF FF FF FF FF");
        assert_snapshot!(encode(f64::MIN), @"00 10 00 00 00 00 00 00");
        assert_snapshot!(encode(f64::MIN_POSITIVE), @"80 10 00 00 00 00 00 00");
    }
}
