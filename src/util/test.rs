use std::fmt::Debug;

use crate::LexOrd;

pub fn test_format<T: LexOrd + Debug>(value: &T, buf: &[u8]) {
    let mut buf_from_value = vec![];
    value.to_write(&mut buf_from_value).unwrap();
    assert!(
        buf == buf_from_value,
        "{value:#?} -> {buf_from_value:x?} != {buf:x?}"
    );
    let value_from_buf = T::from_read(&mut buf_from_value.as_slice()).unwrap();
    assert!(
        value == &value_from_buf,
        "{buf:x?} -> {value_from_buf:?} != {value:?}"
    );
}

pub fn test_write_read<T: LexOrd + Debug>(t: impl Iterator<Item = T>) {
    let mut last = None;
    let mut last_buf = vec![];
    for next in t {
        assert!(
            last.is_none() || last.as_ref().unwrap() < &next,
            "{last:?} >= {next:?}"
        );
        let mut buf = vec![];
        next.to_write(&mut buf).unwrap();
        assert!(
            last_buf < buf,
            "{last:?} >= {next:?} ({last_buf:#?} >= {buf:#?})"
        );
        let mut buf_slice = buf.as_slice();
        let next_reser = T::from_read(&mut buf_slice).unwrap();
        assert!(buf_slice.is_empty(), "({buf:?})");
        assert_eq!(next, next_reser, "({buf:?})");
        last = Some(next);
        last_buf = buf;
    }
}
