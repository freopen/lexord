use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

pub fn derive_lexord(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    let derives = match input.data {
        syn::Data::Struct(data) => derive_struct(name, data),
        syn::Data::Enum(data) => derive_enum(name, data),
        _ => unimplemented!(),
    };
    quote! {
        const _: () = {
            #derives
        };
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
        #[automatically_derived]
        impl PartialEq for #name
        where
            #( #types: PartialEq ),*
        {
            fn eq(&self, other: &Self) -> bool {
                #( (self.#fields == other.#fields) && )* true
            }
        }

        #[automatically_derived]
        impl PartialOrd for #name
        where
            #( #types: PartialOrd ),*
        {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                #( match <#types as PartialOrd>::partial_cmp(&self.#fields, &other.#fields)? {
                    std::cmp::Ordering::Equal => {}
                    ordering => { return Some(ordering); }
                } )*
                Some(std::cmp::Ordering::Equal)
            }
        }

        #[automatically_derived]
        impl ::lexord::LexOrdSer for #name
        where
            #( #types: ::lexord::LexOrdSer ),*
        {
            const OBJECT_TYPE: ::lexord::ObjectType =
                ::lexord::ObjectType::sequence_type(&[
                    #(<#types as ::lexord::LexOrdSer>::OBJECT_TYPE,)*
                ]);

            fn to_write<W: std::io::Write>(&self, writer: &mut W) -> ::lexord::Result {
                #( <#types as ::lexord::LexOrdSer>::to_write(&self.#fields, writer)?; )*
                Ok(())
            }
        }

        #[automatically_derived]
        impl ::lexord::LexOrd for #name
        where
            #( #types: ::lexord::LexOrd ),*
        {
            fn from_read<R: std::io::Read>(reader: &mut R) -> ::lexord::Result<Self> {
                Ok(#name {
                    #( #fields: <#types as ::lexord::LexOrd>::from_read(reader)?, )*
                })
            }
        }
    }
}

fn derive_enum(name: syn::Ident, data: syn::DataEnum) -> TokenStream {
    let mut all_types = HashSet::new();
    let mut eq_hands = vec![];
    let mut cmp_hands = vec![];
    let mut write_hands = vec![];
    let mut read_hands = vec![];

    for (var_index, variant) in data.variants.iter().enumerate() {
        assert!(variant.discriminant.is_none());
        let var_name = &variant.ident;
        let mut field_types = vec![];
        let mut field_names = vec![];
        let mut a_field_names = vec![];
        let mut b_field_names = vec![];
        for (index, field) in variant.fields.iter().enumerate() {
            all_types.insert(&field.ty);
            field_types.push(&field.ty);
            let field_name = match &field.ident {
                Some(ident) => quote! { #ident },
                None => {
                    let index = syn::Index::from(index);
                    quote! { #index }
                }
            };
            field_names.push(quote!(#field_name));
            a_field_names.push(format_ident!("a_{field_name}"));
            b_field_names.push(format_ident!("b_{field_name}"));
        }
        eq_hands.push(quote! {
            (
                #name::#var_name{ #( #field_names: #a_field_names, )* },
                #name::#var_name{ #( #field_names: #b_field_names, )* }
            ) => {
                #( #a_field_names == #b_field_names && )* true
            }
        });
        cmp_hands.push(quote! {
            (
                #name::#var_name{ #( #field_names: #a_field_names, )* },
                #name::#var_name{ #( #field_names: #b_field_names, )* }
            ) => {
                #(
                    match <#field_types as PartialOrd>::partial_cmp(
                        &#a_field_names,
                        &#b_field_names
                    )? {
                        std::cmp::Ordering::Equal => {}
                        ordering => { return Some(ordering); }
                    };
                )*
                return Some(std::cmp::Ordering::Equal);
            }
            (_, #name::#var_name{ .. }) => Some(std::cmp::Ordering::Greater),
            (#name::#var_name{ .. }, _) => Some(std::cmp::Ordering::Less),
        });
        write_hands.push(quote! {
            #name::#var_name{ #( #field_names: #a_field_names, )* } => {
                <usize as ::lexord::LexOrdSer>::to_write(&#var_index, writer)?;
                #( <#field_types as ::lexord::LexOrdSer>::to_write(&#a_field_names, writer)?; )*
            }
        });
        read_hands.push(quote! {
            #var_index => {
                #name::#var_name{ #(
                    #field_names: <#field_types as ::lexord::LexOrd>::from_read(reader)?,
                )* }
            }
        });
    }
    let all_types: Vec<_> = all_types.into_iter().collect();

    quote! {
        #[automatically_derived]
        impl PartialEq for #name
        where
            #( #all_types: PartialEq ),*
        {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #( #eq_hands )*
                    _ => false
                }
            }
        }

        #[automatically_derived]
        impl PartialOrd for #name
        where
            #( #all_types: PartialOrd ),*
        {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                match (self, other) {
                    #( #cmp_hands )*
                }

            }
        }

        #[automatically_derived]
        impl ::lexord::LexOrdSer for #name
        where
            #( #all_types: ::lexord::LexOrdSer ),*
        {
            const OBJECT_TYPE: ::lexord::ObjectType =
                ::lexord::ObjectType::CantStartWithZero;
            fn to_write<W: std::io::Write>(&self, writer: &mut W) -> ::lexord::Result {
                match self {
                    #( #write_hands )*
                }
                Ok(())
            }
        }

        #[automatically_derived]
        impl ::lexord::LexOrd for #name
        where
            #( #all_types: ::lexord::LexOrd ),*
        {
            fn from_read<R: std::io::Read>(reader: &mut R) -> ::lexord::Result<Self> {
                Ok(match <usize as ::lexord::LexOrd>::from_read(reader)? {
                    #( #read_hands )*
                    var_index => {
                        Err(::lexord::Error::Parse(
                            format!("Unexpected enum variant: {var_index}")
                        ))?;
                        unreachable!()
                    }
                })
            }
        }
    }
}
