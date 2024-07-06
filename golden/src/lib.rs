use std::{collections::BTreeSet, fs, path::Path};

use itertools::Itertools;
use lexord::LexOrdSer;
use lexord_fuzz::AnyType;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn generate_goldens_test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(gen_test(parse_macro_input!(input as TokenStream)))
}

fn add_type_recursively(types: &mut BTreeSet<AnyType>, ty: &AnyType) {
    types.insert(ty.clone());
    for child in ty.children.iter() {
        add_type_recursively(types, child);
    }
}

fn gen_test(output: TokenStream) -> TokenStream {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(Path::new("../fuzz/corpus"))
        .canonicalize()
        .unwrap();
    let mut any_types: BTreeSet<AnyType> = Default::default();
    for corpus_item in path.read_dir().unwrap() {
        let raw_data = fs::read(corpus_item.unwrap().path()).unwrap();
        let mut data = arbitrary::Unstructured::new(&raw_data);
        let Ok(any_type) = AnyType::random(&mut data) else {
            continue;
        };
        add_type_recursively(&mut any_types, &any_type);
    }
    let ty = any_types.iter().map(|ty| ty.as_syn()).collect_vec();
    let ser_ty = any_types
        .iter()
        .map(|ty| {
            let mut ser = vec![];
            ty.to_write(&mut ser).unwrap();
            proc_macro2::Literal::byte_string(&ser)
        })
        .collect_vec();
    quote! {
        #(
            test_type::<#ty>(#output, #ser_ty);
        )*
    }
}
