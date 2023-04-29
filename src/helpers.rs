#[cfg(test)]
pub(crate) mod tests {
    use std::fmt::Debug;

    use crate::LexOrd;

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
                "{last:?} >= {next:?} ({last_buf:?} >= {buf:?})"
            );
            let next_reser = T::from_read(&mut buf.as_slice()).unwrap();
            assert_eq!(next, next_reser, "({buf:?})");
            last = Some(next);
            last_buf = buf;
        }
    }
}
