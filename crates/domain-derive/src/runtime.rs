use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use syn::{Ident, Path};

pub fn resolve_runtime() -> Result<Path, syn::Error> {
    match crate_name("alternate-domain") {
        Ok(FoundCrate::Itself) => Ok(syn::parse_quote!(::alternate_domain)),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            Ok(syn::parse_quote!(::#ident))
        }
        Err(error) => Err(syn::Error::new(
            Span::call_site(),
            format!("cannot resolve alternate-domain: {error}"),
        )),
    }
}
