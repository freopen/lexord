use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Index;

pub fn gen_lexord_for_tuples() -> TokenStream {
    let tuple_impls = (1usize..=12usize).map(|tuple_size| {
        let index: Vec<_> = (0..tuple_size).map(Index::from).collect();
        let types: Vec<_> = (0..tuple_size)
            .map(|index| format_ident!("T{}", index))
            .collect();
        quote! {
            impl<#( #types: LexOrdSer ),*> LexOrdSer for ( #( #types, )* ) {
                fn object_type() -> ObjectType {
                    ObjectType::sequence_type(&[#(#types::object_type()),*])
                }

                fn to_write<W: Write>(&self, writer: &mut W) -> Result {
                    #( #types::to_write(&self.#index, writer)?; )*
                    Ok(())
                }
            }

            impl<#( #types: LexOrd ),*> LexOrd for ( #( #types, )* ) {
                fn from_read<R: Read>(reader: &mut R) -> Result<Self> {
                    Ok(( #( #types::from_read(reader)?, )* ))
                }
            }
        }
    });
    quote! {
        #( #tuple_impls )*
    }
}
