use quote::{format_ident, quote};
use syn::Index;

#[proc_macro]
pub fn gen_lexord_for_tuples(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tuple_impls = (1usize..=12usize).map(|tuple_size| {
        let index: Vec<_> = (0..tuple_size).map(Index::from).collect();
        let types: Vec<_> = (0..tuple_size).map(|index| format_ident!("T{}", index)).collect();
        let zero_fns: Vec<_> = (0..tuple_size).map(|index| format_ident!("zero_{}", index)).collect();
        let cant_start_with_zero_hands: Vec<_> = (0..tuple_size).map(|first_non_zero_sized_index| {
            let zero_sized = (0..first_non_zero_sized_index).map(|_| quote! { ObjectType::ZeroSized(_) });
            let placeholders = (0..tuple_size - first_non_zero_sized_index - 1).map(|_| quote! { _ });
            let parts = 
                zero_sized
                    .chain(std::iter::once(quote! { ObjectType::CantStartWithZero }))
                    .chain(placeholders);
            quote! {
                (#( #parts, )*) => ObjectType::CantStartWithZero,
            }
        }).collect();
        let zero_fn_name = format_ident!("zero_fn_{}", tuple_size);
        quote! {
            fn #zero_fn_name<#( #types: LexOrdSer ),*>() -> (#( #types, )*) {
                ( 
                    #( match #types::OBJECT_TYPE {
                        ObjectType::ZeroSized(zero_fn) => zero_fn(),
                        _ => panic!("Not a ZeroSized tuple"),
                    }, )* 
                )
            }
            impl<#( #types: LexOrdSer ),*> LexOrdSer for ( #( #types, )* ) {
                const OBJECT_TYPE: ObjectType<Self> = match ( #( #types::OBJECT_TYPE, )* ) {
                    (#( ObjectType::ZeroSized( #zero_fns ), )*) => ObjectType::ZeroSized(#zero_fn_name),
                    #( #cant_start_with_zero_hands )*
                    _ => ObjectType::Default,
                };
                
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
    proc_macro::TokenStream::from(quote! {
        #( #tuple_impls )*
    })
}
