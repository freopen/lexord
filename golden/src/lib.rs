use std::{collections::BTreeMap, fs, path::Path};

use itertools::Itertools;
use lexord::LexOrdSer;
use lexord_fuzz::{AnyType, AnyValue};
use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro]
pub fn generate_goldens_test(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(gen_test())
}

struct FuzzInput {
    data: Vec<u8>,
    ser_a: Vec<u8>,
    ser_b: Vec<u8>,
}

fn binary_literal(data: &[u8]) -> TokenStream {
    quote!(&[#(#data),*])
}

fn gen_test() -> TokenStream {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(Path::new("../fuzz/corpus"))
        .canonicalize()
        .unwrap();
    let mut types: BTreeMap<AnyType, Vec<FuzzInput>> = BTreeMap::new();
    for corpus_item in path.read_dir().unwrap() {
        let raw_data = fs::read(corpus_item.unwrap().path()).unwrap();
        let mut data = arbitrary::Unstructured::new(&raw_data);
        let Ok(any_type) = AnyType::random(&mut data) else {
            continue;
        };
        any_type.clone().set_current_type();
        let Ok(a) = data.arbitrary::<AnyValue<0>>() else {
            continue;
        };
        let mut ser_a = vec![];
        a.to_write(&mut ser_a).unwrap();
        let Ok(b) = data.arbitrary::<AnyValue<0>>() else {
            continue;
        };
        let mut ser_b = vec![];
        b.to_write(&mut ser_b).unwrap();

        types.entry(any_type).or_default().push(FuzzInput {
            data: raw_data,
            ser_a,
            ser_b,
        });
    }
    let mut outputs = vec![];
    for (any_type, inputs) in types {
        let ty = any_type.as_syn();
        let data = inputs.iter().map(|i| binary_literal(&i.data)).collect_vec();
        let ser_a = inputs
            .iter()
            .map(|i| binary_literal(&i.ser_a))
            .collect_vec();
        let ser_b = inputs
            .iter()
            .map(|i| binary_literal(&i.ser_b))
            .collect_vec();
        outputs.push(quote! {
            {
                let mut values: Vec<TypeValue<#ty>> = vec![];
                #(
                    parse_fuzz_input(&mut values, #data, #ser_a, #ser_b);
                )*
                test_type(values, &mut output);
            }
        });
    }
    quote! {
        #(
            #outputs
        )*
    }
}
