use proc_macro2::Span;
use syn::{
    Data, DeriveInput, Fields, GenericParam, Generics, Ident, Path, Type, Visibility,
    spanned::Spanned,
};

use super::options::Options;
use crate::runtime::resolve_runtime;

pub struct Model {
    pub ident: Ident,
    pub generics: Generics,
    pub raw: Type,
    pub runtime: Path,
    pub validator: Option<Path>,
    pub error: Type,
    pub deref: bool,
}

impl Model {
    pub fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        Self::parse_with_runtime_resolver(input, resolve_runtime)
    }

    pub(super) fn parse_with_runtime_resolver(
        input: DeriveInput,
        resolve_runtime: impl FnOnce() -> Result<Path, syn::Error>,
    ) -> Result<Self, syn::Error> {
        let options = Options::parse(&input.attrs)?;

        let Data::Struct(data) = input.data else {
            return Err(syn::Error::new(
                input.ident.span(),
                "ValueObject requires a struct",
            ));
        };
        let Fields::Unnamed(fields) = data.fields else {
            return Err(syn::Error::new(
                data.fields.span(),
                "ValueObject requires a newtype tuple struct with exactly one private field",
            ));
        };
        if fields.unnamed.len() != 1 {
            return Err(syn::Error::new(
                fields.span(),
                "ValueObject requires exactly one private field",
            ));
        }

        let field = fields.unnamed.into_iter().next().ok_or_else(|| {
            syn::Error::new(
                input.ident.span(),
                "ValueObject requires exactly one private field",
            )
        })?;
        if !matches!(field.vis, Visibility::Inherited) {
            return Err(syn::Error::new(
                field.vis.span(),
                "ValueObject field must be private (inherited visibility)",
            ));
        }
        for attribute in &field.attrs {
            if attribute.path().is_ident("value_object") {
                return Err(syn::Error::new(
                    attribute.span(),
                    "place `value_object` options on the struct, not its field",
                ));
            }
        }

        let error = match (&options.validate, options.error) {
            (Some(_), Some(error)) => error,
            (Some(path), None) => {
                return Err(syn::Error::new(
                    path.span(),
                    "custom validation requires `error = ErrorType`",
                ));
            }
            (None, Some(error)) => {
                return Err(syn::Error::new(
                    error.span(),
                    "`error` requires validation configuration",
                ));
            }
            (None, None) => syn::parse_quote!(::std::convert::Infallible),
        };

        if is_bare_type_parameter(&field.ty, &input.generics) {
            return Err(syn::Error::new(
                field.ty.span(),
                "ValueObject cannot wrap a bare type parameter; implement ValueObject manually",
            ));
        }
        let runtime = resolve_runtime()?;

        Ok(Self {
            ident: input.ident,
            generics: input.generics,
            raw: field.ty,
            runtime,
            validator: options.validate,
            error,
            deref: options.deref.is_present(),
        })
    }

    pub fn helper_ident(&self, suffix: &str) -> Ident {
        let mut name = format!("__value_object_{suffix}");
        while self.generics.params.iter().any(|param| match param {
            GenericParam::Type(param) => param.ident == name,
            GenericParam::Const(param) => param.ident == name,
            GenericParam::Lifetime(param) => param.lifetime.ident == name,
        }) {
            name.push('_');
        }

        Ident::new(&name, Span::mixed_site())
    }
}

fn is_bare_type_parameter(ty: &Type, generics: &Generics) -> bool {
    match ty {
        Type::Group(group) => is_bare_type_parameter(&group.elem, generics),
        Type::Paren(paren) => is_bare_type_parameter(&paren.elem, generics),
        Type::Path(path) if path.qself.is_none() => path
            .path
            .get_ident()
            .is_some_and(|ident| generics.type_params().any(|param| param.ident == *ident)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::*;

    fn parse(tokens: TokenStream) -> Result<Model, syn::Error> {
        Model::parse_with_runtime_resolver(syn::parse2(tokens)?, || Ok(syn::parse_quote!(crate)))
    }

    #[test]
    fn rejects_unsupported_shapes_and_visibility() {
        for tokens in [
            quote! { enum Bad { A } },
            quote! { union Bad { value: u32 } },
            quote! { struct Bad; },
            quote! { struct Bad(); },
            quote! { struct Bad(u32, u32); },
            quote! { struct Bad(pub u32); },
            quote! { struct Bad(pub(crate) u32); },
        ] {
            assert!(parse(tokens).is_err());
        }
    }

    #[test]
    fn rejects_named_fields() {
        let error = parse(quote! {
            struct Bad { value: String }
        })
        .err()
        .expect("named-field structs must be rejected");

        assert_eq!(
            error.to_string(),
            "ValueObject requires a newtype tuple struct with exactly one private field",
        );
    }

    #[test]
    fn accepts_newtype_wrappers() {
        assert!(
            parse(quote! {
                struct Wrapper(String);
            })
            .is_ok()
        );
    }

    #[test]
    fn validates_error_policy_and_coherence() {
        for tokens in [
            quote! { #[value_object(validate = check)] struct Bad(String); },
            quote! { #[value_object(error = Error)] struct Bad(String); },
            quote! {
                #[value_object(validate = check, error = Error)]
                struct Bad<T>(T);
            },
        ] {
            assert!(parse(tokens).is_err());
        }
    }

    #[test]
    fn rejects_unvalidated_bare_type_parameters() {
        assert!(
            parse(quote! {
                struct Wrapper<T>(T);
            })
            .is_err()
        );
    }

    #[test]
    fn preserves_generics_and_escapes_helper_names() {
        let model = parse(quote! {
            struct Wrapper<'a, T, const __value_object_raw: usize>(Vec<&'a T>) where T: 'a;
        })
        .unwrap();

        assert_eq!(model.generics.params.len(), 3);
        assert!(model.generics.where_clause.is_some());
        assert_eq!(model.helper_ident("raw"), "__value_object_raw_");
    }
}
