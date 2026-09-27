use crate::validator::{self, string::UrlError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, alternate_domain::ValueObject)]
#[value_object(validate = validator::string::url, error = UrlError)]
pub struct Url(String);
