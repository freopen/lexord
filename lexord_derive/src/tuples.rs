use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Index;

pub fn gen_lexord_for_tuples() -> TokenStream {
    let tuple_impls = (1usize..=12usize).map(|tuple_size| {
        let index: Vec<_> = (0..tuple_size).map(Index::from).collect();
        let types: Vec<_> = (0..tuple_size)
            .map(|index| format_ident!("T{}", index))
            .collect();
        let first_type = types.first().unwrap();
        let index_no_first: Vec<_> = (1..tuple_size).map(Index::from).collect();
        let types_no_first: Vec<_> = (1..tuple_size)
            .map(|index| format_ident!("T{}", index))
            .collect();
        quote! {
            impl<#( #types: LexOrdSer ),*> LexOrdSer for ( #( #types, )* ) {
                fn to_write(&self, writer: &mut impl Write) -> Result {
                    #( #types::to_write(&self.#index, writer)?; )*
                    Ok(())
                }
                fn to_write_seq(&self, writer: &mut impl Write) -> Result {
                    #first_type::to_write_seq(&self.0, writer)?;
                    #( #types_no_first::to_write(&self.#index_no_first, writer)?; )*
                    Ok(())
                }
            }

            impl<#( #types: LexOrd ),*> LexOrd for ( #( #types, )* ) {
                fn from_read(reader: &mut impl Read) -> Result<Self> {
                    Ok(( #( #types::from_read(reader)?, )* ))
                }
                fn from_read_seq(first: u8, reader: &mut impl Read) -> Result<Self> {
                    Ok((
                        #first_type::from_read_seq(first, reader)?,
                        #( #types_no_first::from_read(reader)?, )*
                    ))
                }
            }
        }
    });
    quote! {
        #( #tuple_impls )*
    }
}
