use proc_macro2::TokenStream;
use quote::quote;

use super::model::Model;

pub fn expand(model: &Model) -> TokenStream {
    let Model {
        ident,
        raw,
        runtime,
        error,
        ..
    } = model;

    let mut generics = model.generics.clone();
    generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote! {
            #raw: #runtime::AsView
        });
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let view_type = quote! { <#raw as #runtime::AsView>::View };
    let value_ident = model.helper_ident("value");

    let validate = if let Some(path) = &model.validator {
        quote! { (#path)(#value_ident)?; ::std::result::Result::Ok(()) }
    } else {
        quote! { ::std::result::Result::Ok(()) }
    };

    let (constructor, conversion) = construction(model, &generics);

    let deref = model.deref.then(|| {
        quote! {
            impl #impl_generics ::std::ops::Deref for #ident #type_generics #where_clause {
                type Target = #view_type;

                fn deref(&self) -> &Self::Target {
                    <Self as #runtime::AsView>::as_view(self)
                }
            }
        }
    });

    quote! {
        impl #impl_generics #runtime::AsView for #ident #type_generics #where_clause {
            type View = #view_type;

            fn as_view(&self) -> &Self::View {
                <#raw as #runtime::AsView>::as_view(&self.0)
            }
        }

        impl #impl_generics #runtime::ValueObject for #ident #type_generics #where_clause {
            type Raw = #raw;
            type Error = #error;

            fn validate(#value_ident: &Self::View) -> ::std::result::Result<(), Self::Error> {
                #validate
            }

            fn into_raw(self) -> Self::Raw {
                self.0
            }
        }

        impl #impl_generics #ident #type_generics #where_clause {
            #constructor

            pub fn validate(#value_ident: &#view_type) -> ::std::result::Result<(), #error> {
                <Self as #runtime::ValueObject>::validate(#value_ident)
            }

            pub fn as_view(&self) -> &#view_type {
                <Self as #runtime::AsView>::as_view(self)
            }

            pub fn into_raw(self) -> #raw {
                <Self as #runtime::ValueObject>::into_raw(self)
            }
        }

        #conversion
        #deref

        impl #impl_generics ::std::convert::AsRef<#view_type> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &#view_type {
                <Self as #runtime::AsView>::as_view(self)
            }
        }
    }
}

fn construction(model: &Model, generics: &syn::Generics) -> (TokenStream, TokenStream) {
    let Model {
        ident,
        raw,
        runtime,
        error,
        ..
    } = model;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let raw_ident = model.helper_ident("raw");
    if model.validator.is_some() {
        (
            quote! {
                pub fn try_new(#raw_ident: impl ::std::convert::Into<#raw>) -> ::std::result::Result<Self, #error> {
                    let #raw_ident: #raw = ::std::convert::Into::into(#raw_ident);
                    <Self as #runtime::ValueObject>::validate(
                        <#raw as #runtime::AsView>::as_view(&#raw_ident),
                    )?;
                    ::std::result::Result::Ok(Self(#raw_ident))
                }
            },
            quote! {
                impl #impl_generics ::std::convert::TryFrom<#raw> for #ident #type_generics #where_clause {
                    type Error = #error;

                    fn try_from(#raw_ident: #raw) -> ::std::result::Result<Self, Self::Error> {
                        Self::try_new(#raw_ident)
                    }
                }
            },
        )
    } else {
        (
            quote! {
                pub fn new(#raw_ident: impl ::std::convert::Into<#raw>) -> Self {
                    Self(::std::convert::Into::into(#raw_ident))
                }
            },
            quote! {
                impl #impl_generics ::std::convert::From<#raw> for #ident #type_generics #where_clause {
                    fn from(#raw_ident: #raw) -> Self {
                        Self::new(#raw_ident)
                    }
                }
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use syn::{File, ImplItem, Item};

    use super::*;

    fn generated_api(input: TokenStream) -> (Vec<String>, Vec<String>) {
        let model = Model::parse_with_runtime_resolver(syn::parse2(input).unwrap(), || {
            Ok(syn::parse_quote!(crate))
        })
        .unwrap();
        let file: File = syn::parse2(expand(&model)).unwrap();

        let mut methods = Vec::new();
        let mut traits = Vec::new();
        for item in file.items {
            if let Item::Impl(item) = item {
                if let Some((path, _)) = item.trait_ {
                    traits.push(path.segments.last().unwrap().ident.to_string());
                } else {
                    for item in item.items {
                        if let ImplItem::Fn(method) = item {
                            methods.push(method.sig.ident.to_string());
                        }
                    }
                }
            }
        }
        (methods, traits)
    }

    #[test]
    fn generates_only_applicable_constructors_and_conversions() {
        for raw in [
            quote! { String },
            quote! { u32 },
            quote! { custom::Storage },
        ] {
            for validated in [false, true] {
                let validation = validated.then(|| quote! { validate = check, error = Error, });
                let (methods, traits) = generated_api(quote! {
                    #[value_object(#validation)]
                    struct Wrapper(#raw);
                });

                assert_eq!(methods.iter().any(|name| name == "new"), !validated);
                assert_eq!(methods.iter().any(|name| name == "try_new"), validated);
                assert_eq!(traits.iter().any(|name| name == "From"), !validated);
                assert_eq!(traits.iter().any(|name| name == "TryFrom"), validated);
                assert!(traits.iter().any(|name| name == "AsView"));
                assert!(!traits.iter().any(|name| name == "FromStr"));
                for method in ["as_str", "as_slice", "get"] {
                    assert!(!methods.iter().any(|name| name == method));
                }
            }
        }
    }
}
