use alternate_validation::builtin;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, alternate_domain::ValueObject)]
#[value_object(validator = builtin::Url)]
pub struct Url(String);
