use alternate_validation::builtin;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, alternate_domain::ValueObject)]
#[value_object(validator = builtin::EmailAddress, from_str, deref)]
pub struct EmailAddress(String);
