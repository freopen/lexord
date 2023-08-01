use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_lexord(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    match input.data {
        syn::Data::Struct(data) => derive_struct(name, data),
        _ => unimplemented!(),
    }
}

fn derive_struct(name: syn::Ident, data: syn::DataStruct) -> TokenStream {
    let (fields, types): (Vec<_>, Vec<_>) = data
        .fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            let index = syn::Index::from(index);
            let ident = match field.ident {
                Some(ident) => quote! { #ident },
                None => quote! { #index },
            };
            (ident, field.ty)
        })
        .unzip();
    quote! {
        impl PartialEq for #name
        where
            #( #types: PartialEq ),*
        {
            fn eq(&self, other: &Self) -> bool {
                #( (self.#fields == other.#fields) && )* true
            }
        }

        impl PartialOrd for #name
        where
            #( #types: PartialOrd ),*
        {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                #( match #types::partial_cmp(&self.#fields, &other.#fields)? {
                    std::cmp::Ordering::Equal => {}
                    ordering => { return Some(ordering); }
                } )*
                Some(std::cmp::Ordering::Equal)
            }
        }

        impl ::lexord::LexOrdSer for #name
        where
            #( #types: LexOrdSer ),*
        {
            const OBJECT_TYPE: ::lexord::ObjectType =
                ::lexord::ObjectType::sequence_type(&[
                    #(<#types as ::lexord::LexOrdSer>::OBJECT_TYPE,)*
                ]);

            fn to_write<W: std::io::Write>(&self, writer: &mut W) -> ::lexord::Result {
                #( self.#fields.to_write(writer)?; )*
                Ok(())
            }
        }

        impl ::lexord::LexOrd for #name
        where
            #( #types: LexOrd ),*
        {
            fn from_read<R: std::io::Read>(reader: &mut R) -> ::lexord::Result<Self> {
                Ok(#name {
                    #( #fields: <#types>::from_read(reader)?, )*
                })
            }
        }
    }
}
