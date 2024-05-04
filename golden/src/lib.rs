use std::{fs, path::Path};

use lexord_fuzz::TypeDef;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse_macro_input;

#[proc_macro]
pub fn generate_goldens_test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(gen_test(&parse_macro_input!(input as Ident)))
}

fn gen_test(for_each_type_fn: &Ident) -> TokenStream {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(Path::new("../fuzz/corpus"))
        .canonicalize()
        .unwrap();
    let mut typedefs = vec![];
    for corpus_item in path.read_dir().unwrap() {
        let data = fs::read(corpus_item.unwrap().path()).unwrap();
        let mut data = arbitrary::Unstructured::new(&data);
        let Ok(typedef) = data.arbitrary::<TypeDef>() else {
            continue;
        };
        match typedef {
            TypeDef::Single(_) => {}
            _ => {
                continue;
            }
        }
        typedefs.push(typedef);
    }
    typedefs.sort();
    typedefs.dedup();
    let mut hashes = vec![];
    let mut type_idents = vec![];
    let mut value_idents = vec![];
    let mut parse_fn_idents = vec![];
    let mut type_definitions = vec![];
    let mut parse_fn_definitions = vec![];
    for typedef in typedefs {
        let hash = typedef.type_hash();
        let type_ident = format_ident!("T{hash:016x}");
        let value_ident = format_ident!("v_{hash:016x}");
        let parse_fn_ident = format_ident!("parse_{hash:016x}");
        type_definitions.push(typedef.define_type());
        parse_fn_definitions.push(match typedef {
            TypeDef::Single(_) => quote! {
                let mut ser = vec![];
                let val: #type_ident = lexord_fuzz::serialize_type(data, &mut ser)?;
                self.#value_ident.push((val, ser));
                let mut ser = vec![];
                let val: #type_ident = lexord_fuzz::serialize_type(data, &mut ser)?;
                self.#value_ident.push((val, ser));
                Ok(())
            },
            _ => unimplemented!(),
        });
        hashes.push(hash);
        type_idents.push(type_ident);
        value_idents.push(value_ident);
        parse_fn_idents.push(parse_fn_ident);
    }
    {
        let mut hashes = hashes.clone();
        hashes.sort();
        for i in 1..hashes.len() {
            assert_ne!(hashes[i - 1], hashes[i]);
        }
    }
    quote! {
        #(#type_definitions)*

        #[derive(Default)]
        struct Registry {
            types: std::collections::HashMap<u64, TypeDef>,
            #(
                #value_idents: Vec<(#type_idents, Vec<u8>)>,
            )*
        }

        impl Registry {
            #(
                fn #parse_fn_idents(&mut self, data: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<()> {
                    #parse_fn_definitions
                }
            )*
            pub fn add(&mut self, raw: &[u8]) -> arbitrary::Result<()> {
                let mut data = arbitrary::Unstructured::new(&raw);
                let typedef: lexord_fuzz::TypeDef = data.arbitrary()?;
                self.types.insert(typedef.type_hash(), typedef.clone());
                match typedef {
                    lexord_fuzz::TypeDef::Single(_) => {}
                    _ => { return Err(arbitrary::Error::IncorrectFormat); }
                };
                match typedef.type_hash() {
                    #(
                        #hashes => self.#parse_fn_idents(&mut data),
                    )*
                    _ => unreachable!("Unexpected TypeDef hash"),
                }
            }
            pub fn for_each_type(self) {
                #(
                    #for_each_type_fn(&self.types[&#hashes], self.#value_idents);
                )*
            }
        }
    }
}
