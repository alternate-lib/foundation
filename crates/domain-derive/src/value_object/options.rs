use darling::{FromMeta, ast::NestedMeta, util::Flag};
use quote::ToTokens;
use syn::{
    Attribute, Expr, ExprPath, Meta, MetaNameValue, Path, Token, Type,
    parse::{ParseStream, discouraged::Speculative},
};

#[derive(Default, darling::FromMeta)]
pub struct Options {
    pub validate: Option<Path>,
    pub error: Option<Type>,
    #[darling(with = crate::attributes::parse_flag)]
    pub deref: Flag,
}

impl Options {
    pub fn parse(attributes: &[Attribute]) -> syn::Result<Self> {
        let mut items = Vec::new();
        for attribute in attributes {
            if attribute.path().is_ident("value_object") {
                items.extend(attribute.parse_args_with(|input: ParseStream<'_>| {
                    input.parse_terminated(Self::parse_item, Token![,])
                })?);
            }
        }

        Self::from_list(&items).map_err(Into::into)
    }

    fn parse_item(input: ParseStream<'_>) -> syn::Result<NestedMeta> {
        let ahead = input.fork();
        let path = ahead.call(Path::parse_mod_style)?;
        if !ahead.peek(Token![=]) || !(path.is_ident("validate") || path.is_ident("error")) {
            return input.parse();
        }

        input.advance_to(&ahead);
        let eq_token = input.parse()?;

        let value = if path.is_ident("validate") {
            Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: input.parse()?,
            })
        } else {
            Expr::Verbatim(input.parse::<Type>()?.into_token_stream())
        };

        Ok(NestedMeta::Meta(Meta::NameValue(MetaNameValue {
            path,
            eq_token,
            value,
        })))
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::DeriveInput;

    use super::*;

    fn parse(tokens: TokenStream) -> Result<Options, syn::Error> {
        let input = syn::parse2::<DeriveInput>(tokens)?;
        Options::parse(&input.attrs)
    }

    #[test]
    fn accepts_typed_paths_and_generic_types() {
        let options = parse(quote! {
            #[value_object(validate = validate::<T>, error = Error<T>, deref)]
            struct Wrapper<T>(Vec<T>);
        })
        .unwrap();

        assert!(options.validate.is_some());
        assert!(options.error.is_some());
        assert!(options.deref.is_present());
    }

    #[test]
    fn preserves_complex_typed_values() {
        for ty in [
            quote!(Error<T, U>),
            quote!(Error<(T, U), [u8; 4]>),
            quote!(&'a Error<T>),
            quote!(fn(T, U) -> Error<T, U>),
            quote!(<T as Trait<U>>::Error),
        ] {
            let options = parse(quote! {
                #[value_object(validate = checks::validate::<T, U>, error = #ty, deref)]
                struct Wrapper<T, U>(Vec<(T, U)>);
            })
            .unwrap();

            let validator = options.validate.unwrap();
            let error = options.error.unwrap();

            assert_eq!(
                quote!(#validator).to_string(),
                quote!(checks::validate::<T, U>).to_string(),
            );

            let expected: Type = syn::parse2(ty).unwrap();

            assert_eq!(quote!(#error).to_string(), quote!(#expected).to_string());
            assert!(options.deref.is_present());
        }
    }

    #[test]
    fn accepts_split_options_and_ignores_unrelated_attributes() {
        let options = parse(quote! {
            #[allow(dead_code)]
            #[value_object(validate = check)]
            #[value_object(error = Error, deref)]
            struct Wrapper(String);
        })
        .unwrap();

        assert!(options.validate.is_some());
        assert!(options.error.is_some());
        assert!(options.deref.is_present());

        let options = parse(quote! { struct Wrapper(String); }).unwrap();

        assert!(options.validate.is_none());
        assert!(options.error.is_none());
        assert!(!options.deref.is_present());
    }

    #[test]
    fn rejects_unknown_options_and_invalid_syntax() {
        for option in [
            quote!(unknown),
            quote!(deref = true),
            quote!(deref()),
            quote!(validate),
            quote!(validate = "check"),
            quote!(validate = check()),
            quote!(error),
            quote!(error = "Error"),
            quote!(error = 42),
        ] {
            assert!(
                parse(quote! {
                    #[value_object(#option)]
                    struct Wrapper(String);
                })
                .is_err(),
                "accepted {option}",
            );
        }
    }

    #[test]
    fn accumulates_unknown_option_errors() {
        let error = parse(quote! {
            #[value_object(first_unknown, second_unknown)]
            struct Wrapper(String);
        })
        .err()
        .expect("unknown options must be rejected");

        assert_eq!(error.into_iter().count(), 2);
    }

    #[test]
    fn rejects_duplicates_within_and_across_attributes() {
        for option in [
            quote!(validate = check),
            quote!(error = Error),
            quote!(deref),
        ] {
            for attributes in [
                quote! { #[value_object(#option, #option)] },
                quote! { #[value_object(#option)] #[value_object(#option)] },
            ] {
                assert!(
                    parse(quote! {
                        #attributes
                        struct Wrapper(String);
                    })
                    .is_err(),
                    "accepted {attributes}",
                );
            }
        }
    }
}
