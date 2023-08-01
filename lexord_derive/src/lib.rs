mod derive;
mod tuples;

#[proc_macro]
pub fn gen_lexord_for_tuples(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(tuples::gen_lexord_for_tuples())
}

#[proc_macro_derive(LexOrd)]
pub fn derive_lexord(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(derive::derive_lexord(syn::parse_macro_input!(
        input as syn::DeriveInput
    )))
}
