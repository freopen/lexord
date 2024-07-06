use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{fold::Fold, parse_macro_input, parse_quote, punctuated::Punctuated};

#[derive(Default)]
struct InferToAnyValue(usize);

impl Fold for InferToAnyValue {
    fn fold_type(&mut self, ty: syn::Type) -> syn::Type {
        match ty {
            syn::Type::Infer(_) => {
                let i = self.0;
                self.0 += 1;
                parse_quote!(AnyValue<#i>)
            }
            _ => syn::fold::fold_type(self, ty),
        }
    }
}

fn define_anyvalue_impl(input: Punctuated<syn::Type, syn::Token![,]>) -> TokenStream {
    let (mut raw_ty, mut ty, mut i, mut variant, mut num_params) =
        (vec![], vec![], vec![], vec![], vec![]);
    for (index, input_ty) in input.into_iter().enumerate() {
        let mut fill = InferToAnyValue::default();
        raw_ty.push(input_ty.clone());
        ty.push(fill.fold_type(input_ty));
        i.push(index as u16);
        variant.push(format_ident!("V{index}"));
        num_params.push(fill.0);
    }

    quote! {
        const NUM_PARAMS: &'static [usize] = &[ #(#num_params),* ];

        impl AnyType {
            pub fn as_syn(&self) -> syn::Type {
                let mut fill = InferToType(0, &self.children);
                match self.type_id {
                    #(#i => <InferToType as syn::fold::Fold>::fold_type(&mut fill, syn::parse_quote!(#raw_ty)),)*
                    _ => unreachable!(),
                }
            }
        }

        #[derive(Debug, PartialEq, PartialOrd)]
        enum AnyValueEnum {
            #(#variant(#ty),)*
        }

        impl<'a, const CHILD_INDEX: usize> arbitrary::Arbitrary<'a> for AnyValue<CHILD_INDEX> {
            fn arbitrary(data: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                log_for_golden(data, CHILD_INDEX);
                Self::with(|type_id: u16| {
                    Ok(Self(Box::new(
                        match type_id {
                            #(#i => AnyValueEnum::#variant(<#ty as arbitrary::Arbitrary<'a>>::arbitrary(data)?),)*
                            _ => unreachable!(),
                        }
                    )))
                })
            }
        }

        impl<const CHILD_INDEX: usize> lexord::LexOrdSer for AnyValue<CHILD_INDEX> {
            fn object_type() -> lexord::ObjectType {
                Self::with(|type_id: u16| {
                    match type_id {
                        #(#i => <#ty as lexord::LexOrdSer>::object_type(),)*
                        _ => unreachable!(),
                    }
                })
            }
            fn to_write<W: std::io::Write>(&self, writer: &mut W) -> lexord::Result {
                Self::with(|type_id: u16| {
                    match &*self.0 {
                        #(AnyValueEnum::#variant(value) => <#ty as lexord::LexOrdSer>::to_write(value,writer),)*
                    }
                })
            }
        }

        impl<const CHILD_INDEX: usize> lexord::LexOrd for AnyValue<CHILD_INDEX> {
            fn from_read<R: std::io::Read>(reader: &mut lexord::PrefixRead<R>) -> lexord::Result<Self> {
                Self::with(|type_id: u16| {
                    match type_id {
                        #(#i => Ok(Self(Box::new(AnyValueEnum::#variant(<#ty as lexord::LexOrd>::from_read(reader)?)))),)*
                        _ => unreachable!(),
                    }
                })
            }
        }
    }
}

#[proc_macro]
pub fn define_anyvalue(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(define_anyvalue_impl(
        parse_macro_input!(input with Punctuated::<syn::Type, syn::Token![,]>::parse_terminated),
    ))
}
