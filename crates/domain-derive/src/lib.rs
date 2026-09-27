use proc_macro::TokenStream;
use syn::DeriveInput;

mod attributes;
mod runtime;
mod value_object;

#[proc_macro_derive(ValueObject, attributes(value_object))]
pub fn derive_value_object(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match value_object::Model::parse(input) {
        Ok(model) => value_object::expand(&model).into(),
        Err(error) => error.into_compile_error().into(),
    }
}
