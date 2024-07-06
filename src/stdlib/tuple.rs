use std::io::{Read, Write};

use lexord_derive::gen_lexord_for_tuples;

use crate::{LexOrd, LexOrdSer, ObjectType, PrefixRead, Result};

impl LexOrdSer for () {
    fn object_type() -> ObjectType {
        ObjectType::ZeroSized
    }

    fn to_write<W: Write>(&self, _writer: &mut W) -> Result {
        Ok(())
    }
}

impl LexOrd for () {
    fn from_read<R: Read>(_reader: &mut PrefixRead<R>) -> Result<Self> {
        Ok(())
    }
}

gen_lexord_for_tuples!();

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::util::test::encode;

    #[test]
    fn test_tuple() {
        assert_snapshot!(encode(()), @"");
        assert_snapshot!(encode((1u8, 2u8, 3u8)), @"01 02 03");
        assert_snapshot!(encode(((), ((), 1u8, ()), ((), (), ()), 2u8, 3u8)), @"01 02 03");
        assert_snapshot!(encode(("abc".to_string(), 2u8, 3u8)), @"61 62 63 00 02 03");
    }
}
