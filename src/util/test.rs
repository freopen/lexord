use std::{cmp::Ordering, fmt::Debug};

use crate::LexOrd;

pub fn encode<T: LexOrd + Debug>(value: T) -> String {
    let mut bytes = vec![];
    value.to_write(&mut bytes).unwrap();
    let mut bytes_slice = bytes.as_slice();
    let value_from_buf = T::from_read(&mut bytes_slice).unwrap();
    assert_eq!(bytes_slice, &[], "Buffer is not consumed completely");
    if let Some(cmp) = value.partial_cmp(&value_from_buf) {
        assert_eq!(
            cmp,
            Ordering::Equal,
            "{bytes:x?} -> {value_from_buf:?} != {value:?}"
        );
    }
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<String>>()
        .join(" ")
}
