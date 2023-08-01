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
    use crate::util::test::{test_format, test_write_read};

    #[test]
    fn test_f32() {
        test_format(&0f32, &[0x80, 0x00, 0x00, 0x00]);
        test_format(&-0f32, &[0x7F, 0xFF, 0xFF, 0xFF]);
        test_format(&1f32, &[0xBF, 0x80, 0x00, 0x00]);
        test_format(&-1f32, &[0x40, 0x7F, 0xFF, 0xFF]);
        test_write_read(
            [
                -1.624235e20f32,
                -1.4e10f32,
                -1f32,
                -0f32,
                1e-21f32,
                0.1209123f32,
                1f32,
                1e10f32,
                1e20f32,
            ]
            .into_iter(),
        );
    }

    #[test]
    fn test_f64() {
        test_format(&0f64, &[0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        test_format(&-0f64, &[0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        test_format(&1f64, &[0xBF, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        test_format(&-1f64, &[0x40, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        test_write_read(
            [
                -1e100f64,
                -1.624235e20f64,
                -1.4e10f64,
                -1f64,
                -0f64,
                1e-21f64,
                0.1209123f64,
                1f64,
                1e10f64,
                1e20f64,
                1e100f64,
            ]
            .into_iter(),
        );
    }
}
