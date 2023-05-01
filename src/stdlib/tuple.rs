use std::io::{Read, Write};

use lexord_derive::gen_lexord_for_tuples;

use crate::{LexOrd, LexOrdSer, ObjectType, Result};

impl LexOrdSer for () {
    const OBJECT_TYPE: ObjectType<Self> = ObjectType::ZeroSized(|| ());

    fn to_write<W: Write>(&self, _writer: &mut W) -> Result {
        Ok(())
    }
}

impl LexOrd for () {
    fn from_read<R: Read>(_reader: &mut R) -> Result<Self> {
        Ok(())
    }
}

gen_lexord_for_tuples!();

#[cfg(test)]
mod tests {
    use crate::helpers::tests::test_format;

    #[test]
    fn test_tuple() {
        test_format(&(), &[]);
        test_format(&(1u8, 2u8, 3u8), &[0x01, 0x02, 0x03]);
        test_format(
            &((), ((), 1u8, ()), ((), (), ()), 2u8, 3u8),
            &[0x01, 0x02, 0x03],
        );
        test_format(
            &("abc".to_string(), 2u8, 3u8),
            &[0x61, 0x62, 0x63, 0x00, 0x02, 0x03],
        );
    }
}
