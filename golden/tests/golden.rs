use std::{
    cell::RefCell,
    fmt::{Debug, Write},
    fs,
    path::Path,
};

use arbitrary::Arbitrary;
use golden::generate_goldens_test;
use itertools::Itertools;
use lexord::LexOrd;
use lexord_fuzz::AnyType;

struct TypeValue<T> {
    type_descriptor: AnyType,
    value: T,
    ser: Vec<u8>,
}

fn parse_fuzz_input<'a, T: Debug + LexOrd + Arbitrary<'a>>(
    output: &mut Vec<TypeValue<T>>,
    data: &'a [u8],
    ser_a: &[u8],
    ser_b: &[u8],
) {
    let mut data = arbitrary::Unstructured::new(data);
    let type_descriptor = AnyType::random(&mut data).unwrap();
    let a: T = data.arbitrary().unwrap();
    let b: T = data.arbitrary().unwrap();
    output.push(TypeValue {
        type_descriptor: type_descriptor.clone(),
        value: a,
        ser: ser_a.to_vec(),
    });
    output.push(TypeValue {
        type_descriptor,
        value: b,
        ser: ser_b.to_vec(),
    })
}

fn test_type<'a, T: Debug + LexOrd + Arbitrary<'a>, W: Write>(
    mut values: Vec<TypeValue<T>>,
    f: &mut W,
) {
    values.sort_unstable_by(|a, b| a.ser.cmp(&b.ser));
    for i in 1..values.len() {
        let a = &values[i - 1];
        let b = &values[i];
        if let Some(ordering) = a.value.partial_cmp(&b.value) {
            assert_eq!(ordering, a.ser.cmp(&b.ser))
        }
    }
    writeln!(f, "# {}\n", values[0].type_descriptor).unwrap();
    writeln!(f, "| Bytes | Value |").unwrap();
    writeln!(f, "| - | - |").unwrap();
    let mut lines = vec![];
    for TypeValue {
        type_descriptor: _,
        value,
        ser,
    } in values
    {
        let mut reser = vec![];
        value.to_write(&mut reser).unwrap();
        assert_eq!(ser, reser);

        let mut deser_slice = &reser[..];
        let deser = T::from_read(&mut deser_slice).unwrap();
        assert_eq!(deser_slice.len(), 0);
        if let Some(ordering) = value.partial_cmp(&deser) {
            assert_eq!(ordering, std::cmp::Ordering::Equal)
        }
        // if ser.len() > 32 {
        //     continue;
        // }
        let ser_string = ser
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect_vec()
            .join(" ");
        let deser_string = format!("{value:?}")
            .chars()
            .chunks(32)
            .into_iter()
            .map(|chunk| format!("`{}`", chunk.collect::<String>()))
            .collect_vec()
            .join("⮒<br>");
        lines.push(format!("| `{ser_string}` | {deser_string} |"));
    }
    lines.dedup();
    for line in lines {
        writeln!(f, "{line}").unwrap();
    }
}

#[test]
fn test() {
    let mut output = String::new();

    generate_goldens_test!();

    println!("{output}");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("golden.md");
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::write(path, output).unwrap();
    } else {
        let golden = fs::read_to_string(path).unwrap();
        assert_eq!(golden, *output);
    }
}
