use afl::fuzz;

use lexord::{LexOrd, LexOrdSer};
use lexord_fuzz::{AnyType, AnyValue};

fn test_raw_bytes(data: &[u8]) -> arbitrary::Result<()> {
    let mut data = arbitrary::Unstructured::new(data);
    let any_type = AnyType::random(&mut data)?;
    any_type.clone().set_current_type();
    let a: AnyValue<0> = data.arbitrary()?;
    let b: AnyValue<0> = data.arbitrary()?;

    let mut ser_a = vec![];
    a.to_write(&mut ser_a).unwrap();
    let mut ser_b = vec![];
    b.to_write(&mut ser_b).unwrap();

    let deser_a = AnyValue::<0>::from_read(&mut ser_a.as_slice()).unwrap();
    let deser_b = AnyValue::<0>::from_read(&mut ser_b.as_slice()).unwrap();

    if let Some(ordering) = a.partial_cmp(&deser_a) {
        assert_eq!(
            ordering,
            std::cmp::Ordering::Equal,
            "{a:?} -> {ser_a:?} -> {deser_a:?}",
        );
    }

    if let Some(ordering) = b.partial_cmp(&deser_b) {
        assert_eq!(
            ordering,
            std::cmp::Ordering::Equal,
            "{a:?} -> {ser_a:?} -> {deser_a:?}",
        );
    }

    assert_eq!(a.partial_cmp(&b), deser_a.partial_cmp(&deser_b));

    if let Some(ordering) = a.partial_cmp(&b) {
        assert_eq!(
            ordering,
            ser_a.cmp(&ser_b),
            "{any_type}: {a:?} -> {ser_a:?} vs. {b:?} -> {ser_b:?}",
        );
    }

    Ok(())
}

pub fn main() {
    fuzz!(|data| {
        let _ = test_raw_bytes(data);
    });
}
