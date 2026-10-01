use darling::{FromAttributes as _, util::Flag};
use syn::{
    Data, DeriveInput, Fields, GenericArgument, Ident, Path, PathArguments, Type, Visibility,
    spanned::Spanned,
};

use crate::runtime::resolve_runtime;

pub struct Model {
    pub ident: Ident,
    pub visibility: Visibility,
    pub runtime: Path,
    pub fields: Vec<Field>,
    pub id: Type,
    pub id_field: Ident,
    pub events_field: Option<Ident>,
}

pub struct Field {
    pub ident: Ident,
    pub leaf: Type,
    pub category: Category,
    pub container: Container,
}

#[derive(Clone, Copy)]
pub enum Category {
    ValueObject,
    Raw,
    Relation,
    Version,
    Events,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Container {
    Scalar,
    Option,
    Vec,
}

impl Model {
    pub fn parse(input: DeriveInput) -> syn::Result<Self> {
        Self::parse_with_runtime_resolver(input, resolve_runtime)
    }

    pub(super) fn parse_with_runtime_resolver(
        input: DeriveInput,
        resolve: impl FnOnce() -> syn::Result<Path>,
    ) -> syn::Result<Self> {
        let aggregate = EntityOptions::from_attributes(&input.attrs)?
            .aggregate
            .is_present();

        if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
            return Err(syn::Error::new(
                input.generics.span(),
                "Entity does not support generics; implement Entity manually",
            ));
        }
        let Data::Struct(data) = input.data else {
            return Err(syn::Error::new(
                input.ident.span(),
                "Entity requires a named-field struct",
            ));
        };
        let Fields::Named(named) = data.fields else {
            return Err(syn::Error::new(
                data.fields.span(),
                "Entity requires a named-field struct",
            ));
        };

        let configured = named
            .named
            .into_iter()
            .map(|field| {
                let options = FieldOptions::from_attributes(&field.attrs)?;
                Ok((field, options))
            })
            .collect::<syn::Result<Vec<_>>>()?;

        let id_field =
            select_field(&configured, "id", |options| &options.id)?.ok_or_else(|| {
                syn::Error::new(
                    input.ident.span(),
                    "Entity requires an #[entity(id)] field or a field named `id`",
                )
            })?;
        let version_field = select_field(&configured, "version", |options| &options.version)?;
        let events_field = select_field(&configured, "events", |options| &options.events)?;
        if version_field.as_ref() == Some(&id_field)
            || events_field.as_ref() == Some(&id_field)
            || (version_field.is_some() && version_field == events_field)
        {
            return Err(syn::Error::new(
                input.ident.span(),
                "Entity id, version, and events roles must use distinct fields",
            ));
        }

        let mut fields = Vec::new();
        let mut id = None;

        for (field, options) in configured {
            let ident = field.ident.expect("named field");
            let category = options.mapping_category(
                &ident,
                aggregate,
                version_field.as_ref() == Some(&ident),
                events_field.as_ref() == Some(&ident),
            )?;

            let (container, leaf) = field_mapping(&field.ty, category)?;
            if ident == id_field {
                if matches!(category, Category::Relation) || container != Container::Scalar {
                    return Err(syn::Error::new(
                        ident.span(),
                        "Entity id must be a scalar value object or #[entity(raw)] field, not a relation or mapped container",
                    ));
                }

                id = Some(field.ty);
            }

            fields.push(Field {
                ident,
                leaf,
                category,
                container,
            });
        }

        let id = id.expect("selected id field exists");
        if aggregate && (version_field.is_none() || events_field.is_none()) {
            return Err(syn::Error::new(
                input.ident.span(),
                "#[entity(aggregate)] requires version and events fields (use #[entity(version)] and #[entity(events)] or fields named `version` and `events`)",
            ));
        }

        Ok(Self {
            ident: input.ident,
            visibility: input.vis,
            runtime: resolve()?,
            fields,
            id,
            id_field,
            events_field,
        })
    }
}

#[derive(darling::FromAttributes)]
#[darling(attributes(entity))]
struct EntityOptions {
    #[darling(with = crate::attributes::parse_flag)]
    aggregate: Flag,
}

#[derive(darling::FromAttributes)]
#[darling(attributes(entity))]
struct FieldOptions {
    #[darling(with = crate::attributes::parse_flag)]
    id: Flag,
    #[darling(with = crate::attributes::parse_flag)]
    version: Flag,
    #[darling(with = crate::attributes::parse_flag)]
    events: Flag,
    #[darling(with = crate::attributes::parse_flag)]
    raw: Flag,
    #[darling(with = crate::attributes::parse_flag)]
    relation: Flag,
}

impl FieldOptions {
    fn mapping_category(
        &self,
        ident: &Ident,
        aggregate: bool,
        version: bool,
        events: bool,
    ) -> syn::Result<Category> {
        let marker = self.category()?;
        if version || events {
            if !aggregate {
                return Err(syn::Error::new(
                    ident.span(),
                    "bookkeeping fields require #[entity(aggregate)]",
                ));
            }
            if marker.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "bookkeeping fields cannot have mapping options",
                ));
            }
            Ok(if version {
                Category::Version
            } else {
                Category::Events
            })
        } else {
            if aggregate && ident == "version" {
                return Err(syn::Error::new(
                    ident.span(),
                    "field collides with generated snapshot `version`",
                ));
            }

            Ok(marker.unwrap_or(Category::ValueObject))
        }
    }

    fn category(&self) -> syn::Result<Option<Category>> {
        match (self.raw.is_present(), self.relation.is_present()) {
            (true, true) => Err(syn::Error::new(
                self.relation.span(),
                "conflicting Entity options `raw` and `relation`",
            )),
            (true, false) => Ok(Some(Category::Raw)),
            (false, true) => Ok(Some(Category::Relation)),
            (false, false) => Ok(None),
        }
    }
}

fn select_field(
    fields: &[(syn::Field, FieldOptions)],
    role: &str,
    flag: impl Fn(&FieldOptions) -> &Flag,
) -> syn::Result<Option<Ident>> {
    let mut selected = None;
    for (field, options) in fields {
        if flag(options).is_present() {
            if selected.is_some() {
                return Err(syn::Error::new(
                    flag(options).span(),
                    format!("duplicate Entity `{role}` role"),
                ));
            }

            selected.clone_from(&field.ident);
        }
    }

    Ok(selected.or_else(|| {
        fields
            .iter()
            .find_map(|(field, _)| field.ident.as_ref().filter(|ident| *ident == role).cloned())
    }))
}

fn field_mapping(ty: &Type, category: Category) -> syn::Result<(Container, Type)> {
    if !matches!(category, Category::ValueObject | Category::Relation) {
        return Ok((Container::Scalar, ty.clone()));
    }

    let (container, leaf) = mapped_type(ty)?;
    if container != Container::Scalar && mapped_type(leaf)?.0 != Container::Scalar {
        return Err(syn::Error::new(
            ty.span(),
            "nested mapped containers are unsupported; use #[entity(raw)] or implement Entity manually",
        ));
    }

    Ok((container, leaf.clone()))
}

fn mapped_type(ty: &Type) -> syn::Result<(Container, &Type)> {
    match ty {
        Type::Paren(ty) => return mapped_type(&ty.elem),
        Type::Group(ty) => return mapped_type(&ty.elem),
        _ => {}
    }

    let Type::Path(path) = ty else {
        return Ok((Container::Scalar, ty));
    };
    if path.qself.is_some() {
        return Ok((Container::Scalar, ty));
    }

    let names: Vec<_> = path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    let names: Vec<_> = names.iter().map(String::as_str).collect();

    let container = match names.as_slice() {
        ["Option"] if path.path.leading_colon.is_none() => Container::Option,
        ["Vec"] if path.path.leading_colon.is_none() => Container::Vec,
        ["std" | "core", "option", "Option"] => Container::Option,
        ["std" | "alloc", "vec", "Vec"] => Container::Vec,
        _ => return Ok((Container::Scalar, ty)),
    };

    if path
        .path
        .segments
        .iter()
        .take(path.path.segments.len() - 1)
        .any(|segment| !matches!(segment.arguments, PathArguments::None))
    {
        return Err(syn::Error::new(
            ty.span(),
            "expected a standard Option<T> or Vec<T> path",
        ));
    }

    if let PathArguments::AngleBracketed(arguments) =
        &path.path.segments.last().expect("nonempty path").arguments
        && arguments.args.len() == 1
        && let Some(GenericArgument::Type(leaf)) = arguments.args.first()
    {
        return Ok((container, leaf));
    }

    Err(syn::Error::new(
        ty.span(),
        "mapped containers require exactly one type argument",
    ))
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    fn parse(input: proc_macro2::TokenStream) -> syn::Result<Model> {
        Model::parse_with_runtime_resolver(syn::parse2(input)?, || Ok(syn::parse_quote!(crate)))
    }

    #[test]
    fn rejects_invalid_shapes_and_configuration() {
        for input in [
            quote! { struct Bad { value: String } },
            quote! { struct Bad(u64); },
            quote! { struct Bad<T> { id: T } },
            quote! { struct Bad { #[entity(raw, relation)] id: u64 } },
            quote! { #[entity(raw)] struct Bad { id: u64 } },
            quote! { struct Bad { #[entity(raw = true)] id: u64 } },
            quote! { #[entity(aggregate)] struct Bad { id: u64 } },
            quote! { struct Bad { id: u64, version: Version } },
            quote! { struct Bad { id: u64, events: EventList<Event> } },
            quote! { struct Bad { #[entity(version)] revision: Version, id: u64 } },
            quote! { struct Bad { #[entity(events)] pending: EventList<Event>, id: u64 } },
            quote! { struct Bad { #[entity(id)] key: Option<Value> } },
            quote! { struct Bad { #[entity(id, relation)] key: Child } },
            quote! { struct Bad { #[entity(id)] a: Value, #[entity(id)] b: Value } },
            quote! { #[entity(aggregate)] struct Bad {
                id: Value, #[entity(version)] a: Version, #[entity(version)] b: Version,
                events: EventList<Event>
            } },
            quote! { #[entity(aggregate)] struct Bad {
                id: Value, version: Version, #[entity(events)] a: EventList<Event>,
                #[entity(events)] b: EventList<Event>
            } },
            quote! { #[entity(aggregate)] struct Bad {
                #[entity(id, version)] key: Value, events: EventList<Event>
            } },
            quote! { #[entity(aggregate)] struct Bad {
                #[entity(id, events)] key: Value, version: Version
            } },
            quote! { #[entity(aggregate)] struct Bad {
                id: Value, #[entity(version, events)] bookkeeping: Version
            } },
            quote! { #[entity(aggregate)] struct Bad {
                id: Value, #[entity(raw)] version: Version, events: EventList<Event>
            } },
            quote! { #[entity(aggregate)] struct Bad {
                id: Value, version: Version, #[entity(relation)] events: EventList<Event>
            } },
            quote! { #[entity(aggregate)] struct Bad {
                id: Value, version: Value, #[entity(version)] revision: Version,
                events: EventList<Event>
            } },
            quote! { #[entity(aggregate)] struct Bad {
                id: Value, _version: Version, _events: EventList<Event>
            } },
            quote! { struct Bad { id: u64, values: Option<Vec<Value>> } },
            quote! { struct Bad { id: Option<Value> } },
        ] {
            assert!(parse(input.clone()).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn rejects_invalid_attribute_options() {
        for input in [
            quote! { #[entity(relation)] struct Bad { id: u64 } },
            quote! { struct Bad { #[entity(aggregate)] id: u64 } },
            quote! { #[entity(unknown)] struct Bad { id: u64 } },
            quote! { struct Bad { #[entity(unknown)] id: u64 } },
            quote! { #[entity(aggregate = true)] struct Bad { id: u64 } },
            quote! { #[entity(aggregate())] struct Bad { id: u64 } },
            quote! { struct Bad { #[entity(raw())] id: u64 } },
            quote! { struct Bad { #[entity(relation = true)] id: u64 } },
            quote! { struct Bad { #[entity(relation())] id: u64 } },
            quote! { struct Bad { #[entity(raw, raw)] id: u64 } },
            quote! { struct Bad { #[entity(raw)] #[entity(raw)] id: u64 } },
            quote! { struct Bad { id: u64, #[entity(relation, relation)] child: Child } },
            quote! { struct Bad { id: u64, #[entity(relation)] #[entity(relation)] child: Child } },
            quote! { struct Bad { #[entity(raw)] #[entity(relation)] id: u64 } },
            quote! { #[entity(aggregate, aggregate)] struct Bad {
                id: u64, version: Version, events: EventList<Event>
            } },
            quote! { #[entity(aggregate)] #[entity(aggregate)] struct Bad {
                id: u64, version: Version, events: EventList<Event>
            } },
            quote! { struct Bad { #[entity(id = true)] id: u64 } },
            quote! { struct Bad { #[entity(id())] id: u64 } },
            quote! { struct Bad { #[entity(id, id)] id: u64 } },
            quote! { struct Bad { #[entity(id)] #[entity(id)] id: u64 } },
            quote! { #[entity(aggregate)] struct Bad {
                id: u64, #[entity(version = true)] version: Version, events: EventList<Event>
            } },
            quote! { #[entity(aggregate)] struct Bad {
                id: u64, version: Version, #[entity(events())] events: EventList<Event>
            } },
        ] {
            assert!(parse(input.clone()).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn recognizes_aggregate_and_mapping_flags() {
        let model = parse(quote! {
            #[allow(dead_code)]
            #[entity(aggregate)]
            struct Good {
                #[entity(raw)] id: u64,
                #[entity(relation)] child: Child,
                value: Value,
                version: Version,
                events: EventList<Event>,
            }
        })
        .unwrap();

        assert_eq!(model.id_field, "id");
        assert_eq!(model.events_field.as_ref().unwrap(), "events");
        assert!(matches!(model.fields[0].category, Category::Raw));
        assert!(matches!(model.fields[1].category, Category::Relation));
        assert!(matches!(model.fields[2].category, Category::ValueObject));
        assert!(matches!(model.fields[3].category, Category::Version));
        assert!(matches!(model.fields[4].category, Category::Events));
    }

    #[test]
    fn recognizes_custom_roles_and_explicit_markers_override_fallbacks() {
        let model = parse(quote! {
            #[entity(aggregate)]
            struct Good {
                #[entity(id, raw)] key: u64,
                #[entity(raw)] id: String,
                #[entity(version)] revision: Version,
                #[entity(events)] pending: EventList<Event>,
                #[entity(raw)] events: String,
            }
        })
        .unwrap();

        assert_eq!(model.id_field, "key");
        assert_eq!(model.events_field.unwrap(), "pending");
        assert!(matches!(model.fields[0].category, Category::Raw));
        assert!(matches!(model.fields[1].category, Category::Raw));
        assert!(matches!(model.fields[2].category, Category::Version));
        assert!(matches!(model.fields[3].category, Category::Events));
        assert!(matches!(model.fields[4].category, Category::Raw));
    }

    #[test]
    fn explicit_roles_can_use_default_names() {
        let model = parse(quote! {
            #[entity(aggregate)]
            struct Good {
                #[entity(id)] id: Value,
                #[entity(version)] version: Version,
                #[entity(events)] events: EventList<Event>,
            }
        })
        .unwrap();

        assert_eq!(model.id_field, "id");
        assert_eq!(model.events_field.unwrap(), "events");
        assert!(matches!(model.fields[0].category, Category::ValueObject));
        assert!(matches!(model.fields[1].category, Category::Version));
        assert!(matches!(model.fields[2].category, Category::Events));
    }

    #[test]
    fn recognizes_only_standard_containers_and_keeps_raw_fields_opaque() {
        let model = parse(quote! {
            struct Good {
                #[entity(raw)] id: u64,
                a: ::core::option::Option<Value>,
                b: std::vec::Vec<Value>,
                c: custom::Option<Value>,
                #[entity(raw)] d: Option<Vec<String>>,
            }
        })
        .unwrap();

        assert!(model.fields[1].container == Container::Option);
        assert!(model.fields[2].container == Container::Vec);
        assert!(model.fields[3].container == Container::Scalar);
        assert!(model.fields[4].container == Container::Scalar);
    }
}
