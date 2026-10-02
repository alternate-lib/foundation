use darling::{FromMeta, ast::NestedMeta, util::Flag};
use quote::ToTokens;
use syn::{
    Attribute, Expr, Meta, MetaNameValue, Path, Token, Type,
    parse::{ParseStream, discouraged::Speculative},
};

#[derive(Default, darling::FromMeta)]
pub struct Options {
    pub validator: Option<Type>,
    pub validator_with: Option<Expr>,
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
        if !ahead.peek(Token![=])
            || !(path.is_ident("validator") || path.is_ident("validator_with"))
        {
            return input.parse();
        }

        input.advance_to(&ahead);
        let eq_token = input.parse()?;

        let value = if path.is_ident("validator") {
            Expr::Verbatim(input.parse::<Type>()?.into_token_stream())
        } else {
            input.parse::<Expr>()?
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
    fn accepts_types_and_construction_expressions() {
        let options = parse(quote! {
            #[value_object(
                validator = checks::Validator<T, U>,
                validator_with = checks::Validator::<T, U>::new(1, 2),
                deref,
            )]
            struct Wrapper<T, U>(Vec<(T, U)>);
        })
        .unwrap();

        let validator = options.validator.unwrap();
        let construction = options.validator_with.unwrap();
        assert_eq!(
            quote!(#validator).to_string(),
            quote!(checks::Validator<T, U>).to_string(),
        );
        assert_eq!(
            quote!(#construction).to_string(),
            quote!(checks::Validator::<T, U>::new(1, 2)).to_string(),
        );
        assert!(options.deref.is_present());
    }

    #[test]
    fn accepts_split_options_and_ignores_unrelated_attributes() {
        let options = parse(quote! {
            #[allow(dead_code)]
            #[value_object(validator = Check)]
            #[value_object(validator_with = Check::new(), deref)]
            struct Wrapper(String);
        })
        .unwrap();

        assert!(options.validator.is_some());
        assert!(options.validator_with.is_some());
        assert!(options.deref.is_present());

        let options = parse(quote! { struct Wrapper(String); }).unwrap();

        assert!(options.validator.is_none());
        assert!(options.validator_with.is_none());
        assert!(!options.deref.is_present());
    }

    #[test]
    fn rejects_unknown_options_and_invalid_syntax() {
        for option in [
            quote!(unknown),
            quote!(deref = true),
            quote!(deref()),
            quote!(validator),
            quote!(validator = "Check"),
            quote!(validator = Check()),
            quote!(validator = 42),
            quote!(validator_with),
            quote!(validate = check),
            quote!(error = Error),
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
    fn rejects_duplicates_within_and_across_attributes() {
        for option in [
            quote!(validator = Check),
            quote!(validator_with = Check::new()),
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
