use std::{
    cell::RefCell,
    fmt::{Debug, Write},
    fs,
    path::Path,
};

use golden::generate_goldens_test;
use lexord::LexOrd;
use lexord_fuzz::TypeDef;

thread_local! {
    pub static OUTPUT: RefCell<String> = RefCell::new(String::new());
}

fn for_each_type_fn<T: Debug + LexOrd>(typedef: &TypeDef, mut values: Vec<(T, Vec<u8>)>) {
    values.sort_by(|a, b| a.1.cmp(&b.1));
    for i in 1..values.len() {
        let a = &values[i - 1];
        let b = &values[i];
        if let Some(ordering) = a.0.partial_cmp(&b.0) {
            assert_eq!(ordering, a.1.cmp(&b.1))
        }
    }
    OUTPUT.with_borrow_mut(|output| {
        writeln!(output, "# {typedef:?}\n").unwrap();
        writeln!(output, "| Bytes | Value |").unwrap();
        writeln!(output, "| - | - |").unwrap();
    });
    let mut lines = vec![];
    for (value, ser) in values {
        let mut reser = vec![];
        value.to_write(&mut reser).unwrap();
        assert_eq!(ser, reser);

        let mut deser_slice = &reser[..];
        let deser = T::from_read(&mut deser_slice).unwrap();
        assert_eq!(deser_slice.len(), 0);
        if let Some(ordering) = value.partial_cmp(&deser) {
            assert_eq!(ordering, std::cmp::Ordering::Equal)
        }

        let ser_string = ser
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<Vec<String>>()
            .join(" ");
        lines.push(format!("| `{ser_string}` | `{value:#?}` |"));
    }
    lines.dedup();
    OUTPUT.with_borrow_mut(|output| writeln!(output, "{}", lines.join("\n")).unwrap());
}

generate_goldens_test!(for_each_type_fn);

#[test]
fn test() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(Path::new("../fuzz/corpus"))
        .canonicalize()
        .unwrap();
    let mut registry = Registry::default();
    for corpus_item in path.read_dir().unwrap() {
        let data = fs::read(corpus_item.unwrap().path()).unwrap();
        let _ = registry.add(&data);
    }
    registry.for_each_type();
    OUTPUT.with_borrow(|output| {
        println!("{output}");
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("golden.md");
        if std::env::var("UPDATE_GOLDEN").is_ok() {
            fs::write(path, output).unwrap();
        } else {
            let golden = fs::read_to_string(path).unwrap();
            assert_eq!(golden, *output);
        }
    })
}
