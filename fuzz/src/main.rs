mod types;

use afl::fuzz;

use crate::types::{FuzzType, TypeDef};

fn test_raw_bytes(data: &[u8]) -> arbitrary::Result<()> {
    let mut data = arbitrary::Unstructured::new(data);
    let typedef: TypeDef = data.arbitrary()?;
    let mut ser_a = vec![];
    let mut ser_b = vec![];
    if let Some(ordering) = typedef.test(&mut data, &mut ser_a, &mut ser_b)? {
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
