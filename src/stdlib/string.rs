use std::io::{Read, Write};

use crate::{LexOrd, LexOrdSer, Result};

impl LexOrdSer for str {
    fn to_write(&self, writer: &mut impl Write) -> Result {
        self.as_bytes().to_write(writer)
    }
}

impl LexOrdSer for String {
    fn to_write(&self, writer: &mut impl Write) -> Result {
        self.as_bytes().to_write(writer)
    }
}

impl LexOrd for String {
    fn from_read(reader: &mut impl Read) -> Result<Self> {
        Ok(String::from_utf8(Vec::<u8>::from_read(reader)?)?)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::util::test::encode;

    #[test]
    fn test_string_format() {
        assert_snapshot!(encode("".to_string()), @"00");
        assert_snapshot!(encode("abc".to_string()), @"61 62 63 00");
        assert_snapshot!(encode("\0".to_string()), @"01 00 00");
        assert_snapshot!(encode("\x01".to_string()), @"01 01 00");
    }
}
