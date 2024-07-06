use std::{
    fmt::{Debug, Write},
    fs,
    io::Read,
    path::Path,
};

use arbitrary::Arbitrary;
use golden::generate_goldens_test;
use itertools::Itertools;
use lexord::{LexOrd, LexOrdSer};
use lexord_fuzz::{AnyType, AnyValue};

struct TypeValue<T> {
    value: T,
    ser: Vec<u8>,
}

fn test_type<T: Debug + LexOrd + for<'a> Arbitrary<'a>>(f: &mut impl Write, any_type: &[u8]) {
    let any_type = AnyType::from_read(&mut any_type.into()).unwrap();
    any_type.clone().set_current_type();
    let data_vec = lexord_fuzz::TYPE_TO_UNSTRUCTURED
        .with_borrow_mut(|type_to_unstructured| type_to_unstructured.remove(&any_type));
    let Some(data_vec) = data_vec else {
        return;
    };
    let mut values = data_vec
        .iter()
        .map(|data| {
            let any_value_ser = {
                let mut data = arbitrary::Unstructured::new(data);
                let any_value: AnyValue<0> = data.arbitrary().unwrap();
                let mut ser = vec![];
                any_value.to_write(&mut ser).unwrap();
                ser
            };
            let mut data = arbitrary::Unstructured::new(data);
            let value: T = data.arbitrary().unwrap();
            let mut ser = vec![];
            value.to_write(&mut ser).unwrap();
            assert_eq!(any_value_ser, ser);
            TypeValue { value, ser }
        })
        .collect_vec();
    values.sort_unstable_by(|a, b| a.ser.cmp(&b.ser));
    for i in 1..values.len() {
        let a = &values[i - 1];
        let b = &values[i];
        if let Some(ordering) = a.value.partial_cmp(&b.value) {
            assert_eq!(ordering, a.ser.cmp(&b.ser))
        }
    }
    writeln!(f, "# {}\n", any_type).unwrap();
    writeln!(f, "| Bytes | Value |").unwrap();
    writeln!(f, "| - | - |").unwrap();
    let mut lines = vec![];
    for TypeValue { value, ser } in values {
        let mut reser = vec![];
        value.to_write(&mut reser).unwrap();
        assert_eq!(ser, reser);

        let mut reser_read = reser.as_slice().into();
        let deser = T::from_read(&mut reser_read).unwrap();
        assert!(reser_read.bytes().next().is_none());
        if let Some(ordering) = value.partial_cmp(&deser) {
            assert_eq!(ordering, std::cmp::Ordering::Equal)
        }
        if ser.len() > 32 {
            continue;
        }
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(Path::new("../fuzz/corpus"))
        .canonicalize()
        .unwrap();
    for corpus_item in path.read_dir().unwrap() {
        let raw_data = fs::read(corpus_item.unwrap().path()).unwrap();
        let mut data = arbitrary::Unstructured::new(&raw_data);
        let Ok(any_type) = AnyType::random(&mut data) else {
            continue;
        };
        any_type.set_current_type();
        if data.arbitrary::<AnyValue<0>>().is_err() {
            continue;
        };
        if data.arbitrary::<AnyValue<0>>().is_err() {
            continue;
        };
    }
    let mut output = String::new();

    generate_goldens_test!(&mut output);

    println!("{output}");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("golden.md");
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::write(path, output).unwrap();
    } else {
        let golden = fs::read_to_string(path).unwrap();
        assert_eq!(golden, *output);
    }
}
