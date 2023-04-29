use std::io::{Read, Write};

use crate::{LexOrd, LexOrdSer, Result};

impl LexOrd for String {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(String::from_utf8(Vec::<u8>::from_read(reader)?)?)
    }
}

impl LexOrdSer for &str {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        self.as_bytes().to_write(writer)
    }
}

impl LexOrdSer for String {
    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        self.as_bytes().to_write(writer)
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::tests::test_format;

    #[test]
    fn test_string_format() {
        test_format(&String::new(), &[0x00]);
        test_format(&"123".to_string(), &[0x31, 0x32, 0x33, 0x00]);
    }
}
