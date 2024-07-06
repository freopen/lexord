use std::{cmp::Ordering, fmt::Debug, io::Read};

use crate::LexOrd;

pub fn encode<T: LexOrd + Debug>(value: T) -> String {
    let mut bytes = vec![];
    value.to_write(&mut bytes).unwrap();
    let mut bytes_read = bytes.as_slice().into();
    let value_from_buf = T::from_read(&mut bytes_read).unwrap();
    assert!(
        bytes_read.bytes().next().is_none(),
        "Buffer is not consumed completely"
    );
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
