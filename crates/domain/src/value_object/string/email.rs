use crate::validator::{self, string::EmailError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, alternate_domain::ValueObject)]
#[value_object(validate = validator::string::email, error = EmailError)]
pub struct Email(String);
