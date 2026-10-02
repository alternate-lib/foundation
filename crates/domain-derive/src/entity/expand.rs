use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

use super::model::{Category, Container, Field, Model};

pub fn expand(model: &Model) -> TokenStream {
    let Model {
        ident,
        visibility,
        runtime,
        id,
        id_field,
        events_field,
        ..
    } = model;

    let snapshot_type = format_ident!("{}Snapshot", ident);
    let snapshot = Ident::new("__entity_snapshot", Span::mixed_site());
    let other = Ident::new("__entity_other", Span::mixed_site());
    let state = Ident::new("__entity_state", Span::mixed_site());
    let hasher = Ident::new("__EntityHasher", Span::mixed_site());

    let mut dto = Vec::new();
    let mut save = Vec::new();
    let mut restore = Vec::new();
    for field in &model.fields {
        let name = &field.ident;
        let ty = snapshot_field_type(field, runtime);

        match field.category {
            Category::Version => {
                dto.push(quote! { pub version: ::std::primitive::u64 });
                save.push(quote! { version: *<#runtime::Version as ::std::ops::Deref>::deref(&self.#name) });
                restore.push(quote! { #name: #runtime::Version::restore(#snapshot.version) });
            }
            Category::Events => {
                restore.push(quote! { #name: <#runtime::EventList<_> as ::std::default::Default>::default() });
            }
            _ => {
                dto.push(quote! { pub #name: #ty });
                let saving = save_field(field, runtime);
                let restoring = restore_field(field, model, &snapshot);
                save.push(quote! { #name: #saving });
                restore.push(quote! { #name: #restoring });
            }
        }
    }

    let check_events = events_field.as_ref().map(|events_field| {
        quote! {
            let _: &#runtime::EventList<_> = &self.#events_field;
        }
    });

    quote! {
        #visibility struct #snapshot_type {
            #(#dto,)*
        }

        impl #runtime::Entity for #ident {
            type Id = #id;
            type Snapshot = #snapshot_type;

            fn id(&self) -> Self::Id {
                self.#id_field
            }

            fn snapshot(&self) -> Self::Snapshot {
                #check_events
                #snapshot_type { #(#save,)* }
            }

            fn restore(#snapshot: Self::Snapshot) -> ::std::result::Result<Self, #runtime::EntityRestoreError> {
                ::std::result::Result::Ok(Self { #(#restore,)* })
            }
        }

        impl ::std::cmp::PartialEq for #ident {
            fn eq(&self, #other: &Self) -> ::std::primitive::bool {
                ::std::cmp::PartialEq::eq(
                    &<Self as #runtime::Entity>::id(self),
                    &<Self as #runtime::Entity>::id(#other),
                )
            }
        }

        impl ::std::cmp::Eq for #ident {}

        impl ::std::hash::Hash for #ident {
            fn hash<#hasher: ::std::hash::Hasher>(&self, #state: &mut #hasher) {
                ::std::hash::Hash::hash(&<Self as #runtime::Entity>::id(self), #state);
            }
        }
    }
}

fn snapshot_field_type(field: &Field, runtime: &syn::Path) -> TokenStream {
    let leaf = &field.leaf;
    let ty = match field.category {
        Category::ValueObject => quote! { <#leaf as #runtime::ValueObject>::Raw },
        Category::Relation => quote! { <#leaf as #runtime::Entity>::Snapshot },
        Category::Raw | Category::Events | Category::Version => quote! { #leaf },
    };

    match field.container {
        Container::Scalar => ty,
        Container::Option => quote! { ::std::option::Option<#ty> },
        Container::Vec => quote! { ::std::vec::Vec<#ty> },
    }
}

fn save_field(field: &Field, runtime: &syn::Path) -> TokenStream {
    let name = &field.ident;
    let leaf = &field.leaf;

    let value = Ident::new("__entity_value", Span::mixed_site());

    let conversion = match field.category {
        Category::ValueObject => quote! {
            <#leaf as #runtime::ValueObject>::into_raw(::std::clone::Clone::clone(#value))
        },
        Category::Relation => quote! { <#leaf as #runtime::Entity>::snapshot(#value) },
        _ => return quote! { ::std::clone::Clone::clone(&self.#name) },
    };

    match field.container {
        Container::Scalar => quote! { { let #value = &self.#name; #conversion } },
        Container::Option => quote! { self.#name.as_ref().map(|#value| #conversion) },
        Container::Vec => quote! {
            ::std::iter::Iterator::collect::<::std::vec::Vec<_>>(
                ::std::iter::Iterator::map(self.#name.iter(), |#value| #conversion)
            )
        },
    }
}

fn restore_field(field: &Field, model: &Model, snapshot: &Ident) -> TokenStream {
    let name = &field.ident;
    let leaf = &field.leaf;
    let runtime = &model.runtime;
    let entity = &model.ident;

    let value = Ident::new("__entity_value", Span::mixed_site());
    let error = Ident::new("__entity_error", Span::mixed_site());

    let conversion = match field.category {
        Category::ValueObject => quote! {
            <#leaf as #runtime::ValueObject>::try_new(#value)
        },
        Category::Relation => quote! { <#leaf as #runtime::Entity>::restore(#value) },
        _ => return quote! { #snapshot.#name },
    };

    let conversion = quote! {
        #conversion.map_err(|#error| #runtime::EntityRestoreError::new(
            ::std::stringify!(#entity), ::std::stringify!(#name), #error,
        ))
    };

    match field.container {
        Container::Scalar => quote! { { let #value = #snapshot.#name; #conversion? } },
        Container::Option => quote! { #snapshot.#name.map(|#value| #conversion).transpose()? },
        Container::Vec => quote! {
            ::std::iter::Iterator::collect::<::std::result::Result<::std::vec::Vec<_>, #runtime::EntityRestoreError>>(
                ::std::iter::Iterator::map(
                    ::std::iter::IntoIterator::into_iter(#snapshot.#name),
                    |#value| #conversion,
                )
            )?
        },
    }
}
