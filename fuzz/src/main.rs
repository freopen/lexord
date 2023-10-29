use afl::fuzz;
use arbitrary::{Arbitrary, Unstructured};
use lexord::LexOrd;
use std::cmp::Ordering;

fn serialize_type<'a, T: LexOrd + Arbitrary<'a>>(
    data: &mut Unstructured<'a>,
    ser: &mut Vec<u8>,
) -> arbitrary::Result<T> {
    let result: T = data.arbitrary()?;
    let prev_pos = ser.len();
    result.to_write(ser).unwrap();
    let mut ser_slice = &ser[prev_pos..];
    let deser = T::from_read(&mut ser_slice).unwrap();
    assert_eq!(ser_slice.len(), 0);
    if let Some(ordering) = result.partial_cmp(&deser) {
        assert_eq!(ordering, Ordering::Equal);
    }
    Ok(result)
}

macro_rules! gen_single_fns {
    ($($ty:ty)+) => {
        #[allow(unused_assignments)]
        fn test_single(
            mut type_id: u16,
            data: &mut Unstructured,
            ser_a: &mut Vec<u8>,
            ser_b: &mut Vec<u8>
        ) -> arbitrary::Result<Option<Ordering>> {
            $(
                if type_id == 0 {
                    let a: $ty = serialize_type(data, ser_a)?;
                    let b: $ty = serialize_type(data, ser_b)?;
                    return Ok(a.partial_cmp(&b));
                }
                type_id -= 1;
            )+
            Err(arbitrary::Error::IncorrectFormat)
        }
    }
}

gen_single_fns!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 String);

#[derive(Debug, Arbitrary)]
enum TypeDef {
    Single(u16),
    Vec(Box<TypeDef>),
    Tuple(Vec<TypeDef>),
    Enum(Vec<(i64, TypeDef)>),
}

fn test_typedef(
    typedef: &TypeDef,
    data: &mut Unstructured,
    ser_a: &mut Vec<u8>,
    ser_b: &mut Vec<u8>,
) -> arbitrary::Result<Option<Ordering>> {
    match typedef {
        TypeDef::Single(type_id) => test_single(*type_id, data, ser_a, ser_b),
        _ => Err(arbitrary::Error::IncorrectFormat),
    }
}

fn test_raw_bytes(data: &[u8]) -> arbitrary::Result<()> {
    let mut data = Unstructured::new(data);
    let typedef: TypeDef = data.arbitrary()?;
    let mut ser_a = vec![];
    let mut ser_b = vec![];
    if let Some(ordering) = test_typedef(&typedef, &mut data, &mut ser_a, &mut ser_b)? {
        assert_eq!(
            ordering,
            ser_a.cmp(&ser_b),
            "{typedef:?} {ser_a:?} {ser_b:?}"
        );
    }
    Ok(())
}

pub fn main() {
    fuzz!(|data| {
        let _ = test_raw_bytes(data);
    });
}
