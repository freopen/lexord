use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

pub fn derive_lexord(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    let generics = input.generics;
    let derives = match input.data {
        syn::Data::Struct(data) => derive_struct(name, generics, data),
        syn::Data::Enum(data) => derive_enum(name, generics, data),
        _ => unimplemented!(),
    };
    quote! {
        const _: () = {
            #derives
        };
    }
}

fn derive_struct(name: syn::Ident, generics: syn::Generics, data: syn::DataStruct) -> TokenStream {
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
    let (first_field, rest_fields) = fields.split_first().unwrap();
    let (first_type, rest_types) = types.split_first().unwrap();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #[automatically_derived]
        impl #impl_generics PartialEq for #name #ty_generics #where_clause
        {
            fn eq(&self, other: &Self) -> bool {
                #( (self.#fields == other.#fields) && )* true
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialOrd for #name #ty_generics #where_clause
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
        impl #impl_generics ::lexord::LexOrdSer for #name #ty_generics #where_clause
        {
            fn to_write(&self, writer: &mut impl std::io::Write) -> ::lexord::Result {
                #( <#types as ::lexord::LexOrdSer>::to_write(&self.#fields, writer)?; )*
                Ok(())
            }
            fn to_write_seq(&self, writer: &mut impl std::io::Write) -> ::lexord::Result {
                <#first_type as ::lexord::LexOrdSer>::to_write_seq(&self.#first_field, writer)?;
                #( <#rest_types as ::lexord::LexOrdSer>::to_write(&self.#rest_fields, writer)?; )*
                Ok(())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::lexord::LexOrd for #name #ty_generics #where_clause
        {
            fn from_read(reader: &mut impl std::io::Read) -> ::lexord::Result<Self> {
                Ok(#name {
                    #( #fields: <#types as ::lexord::LexOrd>::from_read(reader)?, )*
                })
            }
            fn from_read_seq(first: u8, reader: &mut impl std::io::Read) -> ::lexord::Result<Self> {
                Ok(#name {
                    #first_field: <#first_type as ::lexord::LexOrd>::from_read_seq(first, reader)?,
                    #( #rest_fields: <#rest_types as ::lexord::LexOrd>::from_read(reader)?, )*
                })
            }
        }
    }
}

fn derive_enum(name: syn::Ident, generics: syn::Generics, data: syn::DataEnum) -> TokenStream {
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

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #[automatically_derived]
        impl #impl_generics PartialEq for #name #ty_generics #where_clause
        {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #( #eq_hands )*
                    _ => false
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialOrd for #name #ty_generics #where_clause
        {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                match (self, other) {
                    #( #cmp_hands )*
                }

            }
        }

        #[automatically_derived]
        impl #impl_generics ::lexord::LexOrdSer for #name #ty_generics #where_clause
        {
            fn to_write(&self, writer: &mut impl std::io::Write) -> ::lexord::Result {
                match self {
                    #( #write_hands )*
                }
                Ok(())
            }
            fn to_write_seq(&self, writer: &mut impl std::io::Write) -> ::lexord::Result {
                self.to_write(writer)
            }
        }

        #[automatically_derived]
        impl #impl_generics ::lexord::LexOrd for #name #ty_generics #where_clause
        {
            fn from_read(reader: &mut impl std::io::Read) -> ::lexord::Result<Self> {
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
            fn from_read_seq(first: u8, reader: &mut impl std::io::Read) -> ::lexord::Result<Self> {
                Self::from_read(&mut <&[u8] as std::io::Read>::chain(&[first], reader))
            }
        }
    }
}
