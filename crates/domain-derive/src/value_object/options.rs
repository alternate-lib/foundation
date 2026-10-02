use darling::{FromMeta, ast::NestedMeta, util::Flag};
use quote::ToTokens;
use syn::{
    Attribute, Expr, MacroDelimiter, Meta, MetaNameValue, Path, Token, Type,
    parse::{Parse, ParseStream, Parser, discouraged::Speculative},
    punctuated::Punctuated,
};

#[derive(Default, darling::FromMeta)]
pub struct Options {
    pub validator: Option<Type>,
    pub validator_with: Option<Expr>,
    pub error: Option<Type>,
    #[darling(default, with = parse_validators)]
    pub validators: Option<Vec<Type>>,
    #[darling(with = crate::attributes::parse_flag)]
    pub deref: Flag,
    #[darling(with = crate::attributes::parse_flag)]
    pub from_str: Flag,
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

        if path.is_ident("validators") && ahead.peek(syn::token::Paren) {
            input.advance_to(&ahead);
            let content;
            let delimiter = syn::parenthesized!(content in input);
            let types = content.parse_terminated(Type::parse, Token![,])?;

            return Ok(NestedMeta::Meta(Meta::List(syn::MetaList {
                path,
                delimiter: MacroDelimiter::Paren(delimiter),
                tokens: types.into_token_stream(),
            })));
        }

        if !ahead.peek(Token![=])
            || !(path.is_ident("validator")
                || path.is_ident("validator_with")
                || path.is_ident("error"))
        {
            return input.parse();
        }

        input.advance_to(&ahead);
        let eq_token = input.parse()?;

        let value = if path.is_ident("validator") || path.is_ident("error") {
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

fn parse_validators(meta: &Meta) -> darling::Result<Option<Vec<Type>>> {
    let Meta::List(list) = meta else {
        return Err(
            darling::Error::custom("expected `validators(ValidatorType, ...)`").with_span(meta),
        );
    };

    let types = Punctuated::<Type, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
    if types.is_empty() {
        return Err(
            darling::Error::custom("`validators(...)` requires at least one validator")
                .with_span(meta),
        );
    }

    Ok(Some(types.into_iter().collect()))
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
    fn accepts_error_types_expression_macros_and_validator_lists() {
        let options = parse(quote! {
            #[value_object(error = errors::Domain<T, U>)]
            #[value_object(validator_with = checks::all!(First, Second), deref)]
            struct Wrapper<T, U>(Vec<(T, U)>);
        })
        .unwrap();
        let error = options.error.unwrap();

        assert_eq!(
            quote!(#error).to_string(),
            quote!(errors::Domain<T, U>).to_string()
        );
        assert!(matches!(options.validator_with, Some(Expr::Macro(_))));

        for list in [
            quote!(checks::First<T, U>),
            quote!(checks::First<T, U>, checks::Second<{ Wrapper::MAX }>,),
        ] {
            let options = parse(quote! {
                #[value_object(validators(#list))]
                #[value_object(error = errors::Domain)]
                struct Wrapper(String);
            })
            .unwrap();
            let validators = options.validators.unwrap();

            assert!(!validators.is_empty());
            assert!(options.error.is_some());
        }
    }

    #[test]
    fn accepts_split_options_and_ignores_unrelated_attributes() {
        let options = parse(quote! {
            #[allow(dead_code)]
            #[value_object(validator = Check)]
            #[value_object(validator_with = Check::new(), deref, from_str)]
            struct Wrapper(String);
        })
        .unwrap();

        assert!(options.from_str.is_present());
        assert!(options.validator.is_some());
        assert!(options.validator_with.is_some());
        assert!(options.deref.is_present());

        let options = parse(quote! { struct Wrapper(String); }).unwrap();

        assert!(options.validator.is_none());
        assert!(options.validator_with.is_none());
        assert!(!options.deref.is_present());
        assert!(!options.from_str.is_present());
    }

    #[test]
    fn rejects_unknown_options_and_invalid_syntax() {
        for option in [
            quote!(unknown),
            quote!(deref = true),
            quote!(deref()),
            quote!(from_str = true),
            quote!(from_str()),
            quote!(validator),
            quote!(validator = "Check"),
            quote!(validator = Check()),
            quote!(validator = 42),
            quote!(validator_with),
            quote!(validate = check),
            quote!(error),
            quote!(error = "Error"),
            quote!(validators),
            quote!(validators = Check),
            quote!(validators()),
            quote!(validators(Check())),
            quote!(validators("Check")),
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
            quote!(from_str),
            quote!(error = Error),
            quote!(validators(Check)),
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
