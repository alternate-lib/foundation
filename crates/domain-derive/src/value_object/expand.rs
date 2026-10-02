use proc_macro2::{Span, TokenStream};
use quote::quote;

use super::model::Model;

pub fn expand(model: &Model) -> TokenStream {
    let Model {
        ident,
        raw,
        runtime,
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
    let raw_ident = model.helper_ident("raw");

    let mut vo_generics = generics.clone();
    vo_generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote! {
            #ident #type_generics: ::std::cmp::PartialEq
        });

    let (error, validate) = validation(model, &view_type, &mut vo_generics);
    let (vo_impl_generics, _, vo_where_clause) = vo_generics.split_for_impl();
    let (constructor, conversion) = construction(model, &vo_generics);

    let from_str = model.from_str.then(|| {
        let mut generics = vo_generics.clone();
        let lifetime = syn::Lifetime::new(
            &format!("'{}", model.helper_ident("input")),
            Span::mixed_site(),
        );

        generics.make_where_clause().predicates.push(syn::parse_quote! {
            #raw: #runtime::AsView<View = str> + for<#lifetime> ::std::convert::From<&#lifetime str>
        });
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        quote! {
            impl #impl_generics ::std::str::FromStr for #ident #type_generics #where_clause {
                type Err = <Self as #runtime::ValueObject>::Error;

                fn from_str(#raw_ident: &str) -> ::std::result::Result<Self, Self::Err> {
                    <Self as #runtime::ValueObject>::try_new(
                        <#raw as ::std::convert::From<&str>>::from(#raw_ident),
                    )
                }
            }
        }
    });

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

        impl #vo_impl_generics #runtime::ValueObject for #ident #type_generics #vo_where_clause {
            type Raw = #raw;
            type Error = #error;

            fn try_new(#raw_ident: Self::Raw) -> ::std::result::Result<Self, Self::Error> {
                #validate
                ::std::result::Result::Ok(Self(#raw_ident))
            }

            fn into_raw(self) -> Self::Raw {
                self.0
            }
        }

        impl #vo_impl_generics #ident #type_generics #vo_where_clause {
            #constructor

            pub fn as_view(&self) -> &#view_type {
                <Self as #runtime::AsView>::as_view(self)
            }

            pub fn into_raw(self) -> #raw {
                <Self as #runtime::ValueObject>::into_raw(self)
            }
        }

        #conversion
        #from_str
        #deref

        impl #impl_generics ::std::convert::AsRef<#view_type> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &#view_type {
                <Self as #runtime::AsView>::as_view(self)
            }
        }
    }
}

fn validation(
    model: &Model,
    view_type: &TokenStream,
    generics: &mut syn::Generics,
) -> (TokenStream, TokenStream) {
    if !model.is_validated() {
        return (quote! { ::std::convert::Infallible }, TokenStream::new());
    }

    let raw = &model.raw;
    let runtime = &model.runtime;
    let raw_ident = model.helper_ident("raw");
    let validator_ident = model.helper_ident("validator");

    let (validator, construction) = if let Some(validators) = &model.validators {
        let (ty, expr) = lower_validators(model, validators, view_type, generics);

        (Some(ty), expr)
    } else {
        let ty = model.validator.as_ref().map(|ty| quote! { #ty });
        let expr = if let Some(expr) = &model.validator_with {
            quote! { #expr }
        } else {
            let ty = ty.as_ref().expect("validated model has a validator");
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote! {
                    #ty: ::std::default::Default
                });

            quote! { <#ty as ::std::default::Default>::default() }
        };

        (ty, expr)
    };

    if let Some(validator) = &validator {
        generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                #validator: #runtime::__private::Validator<#view_type>
            });
    }

    let annotation = validator.as_ref().map(|ty| quote! { : #ty });
    let error = if let Some(error) = &model.error {
        generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                #error: ::std::error::Error
            });

        if let Some(validator) = &validator {
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote! {
                    #error: ::std::convert::From<
                        <#validator as #runtime::__private::Validator<#view_type>>::Error
                    >
                });
        }

        quote! { #error }
    } else {
        let validator = validator
            .as_ref()
            .expect("inferred validator has an explicit error");

        quote! { <#validator as #runtime::__private::Validator<#view_type>>::Error }
    };

    let mapping = model.error.as_ref().map(|error| {
        quote! {
            let #validator_ident = #runtime::__private::MapErr::new(
                #validator_ident,
                <#error as ::std::convert::From<_>>::from,
            );
        }
    });

    (
        quote! { #runtime::__private::ValidationErrors<#error> },
        quote! {
            let #validator_ident #annotation = #construction;
            #mapping
            #runtime::__private::Validator::validate(
                &#validator_ident,
                <#raw as #runtime::AsView>::as_view(&#raw_ident),
            )?;
        },
    )
}

fn lower_validators(
    model: &Model,
    validators: &[syn::Type],
    view_type: &TokenStream,
    generics: &mut syn::Generics,
) -> (TokenStream, TokenStream) {
    let runtime = &model.runtime;
    let error = model
        .error
        .as_ref()
        .expect("validator list has an error type");

    let mut mapped = validators
        .iter()
        .map(|validator| {
            let predicates: [syn::WherePredicate; 2] = [
                syn::parse_quote! {
                    #validator: #runtime::__private::Validator<#view_type> + ::std::default::Default
                },
                syn::parse_quote! {
                    #error: ::std::convert::From<
                        <#validator as #runtime::__private::Validator<#view_type>>::Error
                    >
                },
            ];
            generics.make_where_clause().predicates.extend(predicates);

            let source_error = quote! {
                <#validator as #runtime::__private::Validator<#view_type>>::Error
            };
            let mapper = quote! { fn(#source_error) -> #error };
            (
                quote! { #runtime::__private::MapErr<#validator, #mapper> },
                quote! {
                    #runtime::__private::MapErr::new(
                        <#validator as ::std::default::Default>::default(),
                        <#error as ::std::convert::From<#source_error>>::from as #mapper,
                    )
                },
            )
        })
        .collect::<Vec<_>>();

    let mut result = mapped.pop().expect("validator list is nonempty");
    while let Some((ty, expr)) = mapped.pop() {
        let (rest_ty, rest_expr) = result;
        result = (
            quote! { #runtime::__private::And<#ty, #rest_ty> },
            quote! { #runtime::__private::And::new(#expr, #rest_expr) },
        );
    }

    result
}

fn construction(model: &Model, generics: &syn::Generics) -> (TokenStream, TokenStream) {
    let Model {
        ident,
        raw,
        runtime,
        ..
    } = model;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let raw_ident = model.helper_ident("raw");
    let error_ident = model.helper_ident("error");
    let value_ident = model.helper_ident("value");

    if model.is_validated() {
        (
            quote! {
                pub fn try_new(#raw_ident: impl ::std::convert::Into<#raw>) -> ::std::result::Result<Self, <Self as #runtime::ValueObject>::Error> {
                    <Self as #runtime::ValueObject>::try_new(::std::convert::Into::into(#raw_ident))
                }
            },
            quote! {
                impl #impl_generics ::std::convert::TryFrom<#raw> for #ident #type_generics #where_clause {
                    type Error = <Self as #runtime::ValueObject>::Error;

                    fn try_from(#raw_ident: #raw) -> ::std::result::Result<Self, Self::Error> {
                        <Self as #runtime::ValueObject>::try_new(#raw_ident)
                    }
                }
            },
        )
    } else {
        (
            quote! {
                pub fn new(#raw_ident: impl ::std::convert::Into<#raw>) -> Self {
                    match <Self as #runtime::ValueObject>::try_new(::std::convert::Into::into(#raw_ident)) {
                        ::std::result::Result::Ok(#value_ident) => #value_ident,
                        ::std::result::Result::Err(#error_ident) => match #error_ident {},
                    }
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
    fn generates_from_str_only_when_opted_in() {
        for validation in [quote! {}, quote! { validator = Check, }] {
            let (_, traits) = generated_api(quote! {
                #[value_object(#validation from_str)]
                struct Wrapper(String);
            });

            assert!(traits.iter().any(|name| name == "FromStr"));
        }
    }

    #[test]
    fn generates_only_applicable_constructors_and_conversions() {
        for raw in [
            quote! { String },
            quote! { u32 },
            quote! { custom::Storage },
        ] {
            for validation in [
                quote! {},
                quote! { validator = Check },
                quote! { validator = Check, error = DomainError },
                quote! { validator_with = Check::new(), error = DomainError },
                quote! { validators(First, Second), error = DomainError },
            ] {
                let validated = !validation.is_empty();
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
                for method in ["validate", "as_str", "as_slice", "get"] {
                    assert!(!methods.iter().any(|name| name == method));
                }
            }
        }
    }
}
