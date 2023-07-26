#![no_main]
use lexord::{LexOrd, LexOrdSer};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<f64>| {
    let mut buf_x: Vec<_> = data
        .into_iter()
        .map(|x| {
            let mut buf = vec![];
            x.to_write(&mut buf).unwrap();
            let reser = <f64>::from_read(&mut buf.as_slice()).unwrap();
            if reser.partial_cmp(&x).is_some() {
                assert_eq!(x, reser);
            }
            (buf, x)
        })
        .collect();
    buf_x.sort_by(|x, y| x.0.cmp(&y.0));
    buf_x.into_iter().reduce(|x, y| {
        if let Some(ordering) = x.1.partial_cmp(&y.1) {
            assert_ne!(ordering, std::cmp::Ordering::Greater, "{} > {}", x.1, y.1);
        }
        y
    });
});
