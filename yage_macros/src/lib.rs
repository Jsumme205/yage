use proc_macro::TokenStream;

mod impl_component_convert;
mod utils;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let ty = syn::parse_macro_input!(input as syn::DeriveInput);
    todo!()
}
