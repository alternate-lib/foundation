extern crate self as alternate_domain;

pub use aggregate_root::*;
pub use alternate_domain_derive::{Entity, ValueObject};
pub use entity::*;
pub use query::*;
pub use repository::*;
pub use transaction_scope::*;
pub use use_case::*;
pub use value_object::{AsView, ValueObject};

#[doc(hidden)]
pub mod __private {
    pub use alternate_validation::{And, MapErr, ValidationErrors, Validator};
}

mod aggregate_root;
mod entity;
mod query;
mod repository;
mod transaction_scope;
mod use_case;
pub mod value_object;
